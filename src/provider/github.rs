use std::{
    env::{
        self,
        consts::{ARCH, OS},
    },
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
    config::{LockedPackage, Package, Source},
    context::Context,
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

impl Provider for Github {
    async fn download(&self, ctx: &Context, pkg: &Package) -> Result<LockedPackage> {
        let repo = match &pkg.source {
            Source::Github { repo } => repo,
        };

        let (owner, repo) = repo.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid repo"))?;
        let (version, release) = match &pkg.version {
            Some(version) => {
                let release = self.crab.repos(owner, repo).releases().get_by_tag(version).await?;
                (version.to_owned(), release)
            }
            None => {
                let release = self.crab.repos(owner, repo).releases().get_latest().await?;
                (release.tag_name.to_owned(), release)
            }
        };
        ctx.log_verbose_status("Fetched", &format!("{owner}/{repo}@{version}"));

        let asset = filter_assets(&release)?;
        ctx.log_verbose_status("Filtered", &asset.name);

        let path = ctx.cache_dir.join(&asset.name);

        // skip download if the asset already exists
        if path.exists() {
            ctx.log_verbose_status("Skipped", &"Asset already exists");
        } else {
            self.http
                .download(asset.browser_download_url.clone(), &ctx.cache_dir, &asset.name)
                .await?;
            ctx.log_verbose_status("Downloaded", &asset.browser_download_url);
        }

        // get description from the release if not provided
        let desc = match &pkg.desc {
            Some(desc) => desc.clone().into(),
            None => self
                .crab
                .repos(owner, repo)
                .get()
                .await
                .ok()
                .and_then(|repo| repo.description),
        };

        Ok(LockedPackage {
            name: pkg.name.clone(),
            version,
            source: pkg.source.clone(),
            desc,
            filename: asset.name.clone(),
            download_url: asset.browser_download_url.clone().into(),
        })
    }

    async fn download_locked(&self, ctx: &Context, lpkg: &LockedPackage) -> Result<()> {
        let path = ctx.cache_dir.join(&lpkg.filename);

        // skip download if the asset already exists
        if path.exists() {
            ctx.log_verbose_status("Skipped", &"Asset already exists");
            return Ok(());
        }

        let repo = match &lpkg.source {
            Source::Github { repo } => repo,
        };
        let version = &lpkg.version;

        let download_url = match lpkg.download_url.as_ref() {
            Some(url) => url.clone(),
            None => {
                let (owner, repo) = repo.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid repo"))?;
                let release = self.crab.repos(owner, repo).releases().get_by_tag(version).await?;
                ctx.log_verbose_status("Fetched", &format!("{owner}/{repo}@{version}"));
                let asset = release
                    .assets
                    .iter()
                    .find(|asset| asset.name == lpkg.filename)
                    .ok_or_else(|| anyhow::anyhow!("Asset not found"))?;
                asset.browser_download_url.clone()
            }
        };

        ctx.log_verbose_status("Downloading", &download_url);
        self.http
            .download(download_url.clone(), &ctx.cache_dir, &lpkg.filename)
            .await?;

        Ok(())
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
