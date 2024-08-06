use std::{
    env::{
        self,
        consts::{ARCH, OS},
    },
    path::PathBuf,
    sync::LazyLock,
};

use anyhow::{bail, Result};
use octocrab::{
    models::repos::{Asset, Release},
    Octocrab,
};
use regex::Regex;
use tracing::{trace, warn};

use crate::{
    config::{Package, Source},
    context::Context,
    lock::LockedPackage,
    provider::reqwest_ext::Download,
};

use super::Provider;

pub struct Github {
    crab: Octocrab,
    http: reqwest::Client,
}

impl Github {
    pub fn new() -> Result<Self> {
        let crab = match env::var("GITHUB_TOKEN").or_else(|_| env::var("GRM_GITHUB_TOKEN")) {
            Ok(token) => Octocrab::builder().personal_token(token).build()?,
            Err(_) => Octocrab::builder().build()?,
        };
        let http = reqwest::Client::new();
        Ok(Github { crab, http })
    }
}

impl Provider for &Github {
    async fn download(&self, ctx: Context, pkg: &Package) -> Result<PathBuf> {
        let repo = match &pkg.source {
            Source::Github { repo } => repo,
        };

        let (owner, repo) = repo.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid repo"))?;

        let release = self.crab.repos(owner, repo).releases().get_by_tag(&pkg.version).await?;
        ctx.log_verbose_status("Fetched", &format!("{owner}/{repo}@{version}", version = pkg.version));

        let asset = filter_assets(&release)?;
        ctx.log_verbose_status("Filtered", &asset.name);

        let path = ctx.cache_dir.join(&asset.name);

        // download the asset if not exists
        if path.exists() {
            // TODO: checksum
            ctx.log_verbose_status("Skipped", &"Asset already exists");
        } else {
            ctx.log_verbose_status("Downloading", &asset.browser_download_url);
            self.http.download(asset.browser_download_url.clone(), &path).await?;
        }

        Ok(path)
    }

    async fn download_locked(&self, ctx: Context, pkg: &LockedPackage) -> Result<PathBuf> {
        let path = ctx.cache_dir.join(&pkg.filename);

        // download the asset if not exists
        if path.exists() {
            // TODO: checksum
            ctx.log_verbose_status("Skipped", &"Asset already exists");
            return Ok(path);
        }

        let repo = match &pkg.pkg.source {
            Source::Github { repo } => repo,
        };
        let version = &pkg.pkg.version;

        let (owner, repo) = repo.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid repo"))?;
        let release = self.crab.repos(owner, repo).releases().get_by_tag(version).await?;
        ctx.log_verbose_status("Fetched", &format!("{owner}/{repo}@{version}"));

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == pkg.filename)
            .ok_or_else(|| anyhow::anyhow!("Asset not found"))?;

        ctx.log_verbose_status("Downloading", &asset.browser_download_url);
        self.http.download(asset.browser_download_url.clone(), &path).await?;

        Ok(path)
    }
}

fn filter_assets(release: &Release) -> anyhow::Result<&Asset> {
    let assets = release
        .assets
        .iter()
        .inspect(|asset| {
            trace!("before filter: {asset}", asset = asset.name);
        })
        .filter(|asset| match OS {
            "linux" => asset.name.contains("linux"),
            "macos" => {
                static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#".*(apple|darwin|osx|mac).*"#).unwrap());
                RE.is_match(&asset.name.to_lowercase())
            }
            _ => false,
        })
        .filter(|asset| match ARCH {
            "x86_64" => asset.name.contains("amd64") || asset.name.contains("x86_64"),
            "x86" => asset.name.contains("386") || asset.name.contains("x86"),
            _ => false,
        })
        .filter(|asset| !asset.name.ends_with("checksum") && !asset.name.ends_with("sha256sum"))
        .filter(|asset| !asset.name.ends_with(".sbom"))
        .collect::<Vec<_>>();

    // choose the musl version if available
    let musl_assets = assets
        .iter()
        .filter(|asset| asset.name.contains("musl"))
        .cloned()
        .collect::<Vec<_>>();

    let assets = if !musl_assets.is_empty() { musl_assets } else { assets };

    match &assets[..] {
        [] => bail!("asset not found after filtered"),
        [asset] => Ok(asset),
        [asset, ..] => {
            warn!(
                "{} assets found, the first one will be used: {:#?}",
                assets.len(),
                assets.iter().map(|asset| &asset.name).collect::<Vec<_>>()
            );
            Ok(asset)
        }
    }
}
