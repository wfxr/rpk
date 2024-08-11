use std::env::{
    self,
    consts::{ARCH, OS},
};

use anyhow::{bail, Context as _, Result};
use octocrab::{
    models::{
        repos::{Asset, Release},
        Repository,
    },
    Octocrab,
};
use tracing::{debug, trace, warn};
use url::Url;

use crate::{
    config::{LockedPackage, Package, Source},
    context::Context,
    util::reqwest::Download as _,
};

use super::Provider;

pub struct Github {
    crab:  Octocrab,
    http:  reqwest::Client,
    token: Option<String>,

    ctx: Context,
}

impl Github {
    pub fn new(ctx: Context) -> Result<Self> {
        let builder = Octocrab::builder();

        let token = env::var("GITHUB_TOKEN").or_else(|_| env::var("RPK_GITHUB_TOKEN")).ok();

        let crab = match &token {
            Some(token) => builder.personal_token(token.clone()).build()?,
            None => builder.build()?,
        };

        let http = reqwest::Client::builder().user_agent("rpk").build()?;
        Ok(Github { crab, http, token, ctx })
    }

    pub async fn search_repo(&self, query: &str, size: impl Into<u8>) -> Result<Vec<Repository>> {
        self.crab
            .search()
            .repositories(query)
            .per_page(size)
            .send()
            .await
            .map(|x| x.items)
            .context("failed to search package")
    }

    pub async fn get_release(&self, repo: &str, version: Option<&str>) -> Result<Release> {
        let (owner, repo) = self.parse_repo(repo)?;
        match version {
            Some(version) => self.crab.repos(owner, repo).releases().get_by_tag(version).await,
            None => self.crab.repos(owner, repo).releases().get_latest().await,
        }
        .context(format!(
            "failed to get release: `{owner}/{repo}@{version}`",
            version = version.unwrap_or("latest")
        ))
    }

    pub async fn get_repo(&self, repo: &str) -> Result<Repository> {
        let (owner, repo) = self.parse_repo(repo)?;
        self.crab
            .repos(owner, repo)
            .get()
            .await
            .context(format!("failed to get repo: `{owner}/{repo}`"))
    }

    pub fn parse_repo<'a>(&self, repo: &'a str) -> Result<(&'a str, &'a str)> {
        repo.split_once('/').context(format!("Invalid repo: `{repo}`"))
    }

    pub async fn download_asset(&self, name: &str, url: Url) -> Result<()> {
        self.ctx.log_verbose_status("Downloading", &url);
        self.http
            .download(url, name, &self.ctx.cache_dir, self.token.as_deref())
            .await?;
        self.ctx.log_status("Downloaded", name);
        Ok(())
    }
}

impl Provider for Github {
    async fn download(&self, ctx: &Context, pkg: &Package) -> Result<LockedPackage> {
        let repo = match &pkg.source {
            Source::Github { repo } => repo,
        };

        let release = self.get_release(repo, pkg.version.as_deref()).await?;
        ctx.log_verbose_status("Fetched", format!("{repo}@{version}", version = release.tag_name));

        let asset = filter_assets(&release)?;
        ctx.log_verbose_status("Filtered", &asset.name);

        let path = ctx.cache_dir.join(&asset.name);

        // skip download if the asset already exists
        if path.exists() {
            ctx.log_verbose_status("Skipped", format!("Asset already exists: {}", asset.name));
        } else {
            self.download_asset(&asset.name, asset.browser_download_url.clone())
                .await?;
        }

        // get description from the release if not provided
        let desc = match &pkg.desc {
            Some(desc) => desc.clone().into(),
            None => self.get_repo(repo).await.ok().and_then(|repo| repo.description),
        };

        Ok(LockedPackage {
            name: pkg.name.clone(),
            version: release.tag_name.clone(),
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
            ctx.log_verbose_status("Skipped", format!("Asset already exists: {}", lpkg.filename));
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
                ctx.log_verbose_status("Fetched", format!("{owner}/{repo}@{version}"));
                let asset = release
                    .assets
                    .iter()
                    .find(|asset| asset.name == lpkg.filename)
                    .ok_or_else(|| anyhow::anyhow!("Asset not found"))?;
                asset.browser_download_url.clone()
            }
        };

        self.download_asset(&lpkg.filename, download_url).await?;

        Ok(())
    }
}

fn filter_assets(release: &Release) -> anyhow::Result<&Asset> {
    debug!("OS: {OS}, ARCH: {ARCH}");

    let assets = release
        .assets
        .iter()
        .inspect(|asset| {
            trace!("before filter: {asset}", asset = asset.name);
        })
        .filter(|asset| match OS {
            "linux" => asset.name.contains("linux"),
            "macos" => ["apple", "darwin", "osx", "mac"]
                .iter()
                .any(|os| asset.name.contains(os)),
            _ => false,
        })
        .filter(|asset| match ARCH {
            "x86_64" => is_x86_64(asset),
            "x86" => is_x86(asset),
            "aarch64" => is_aarch64(asset),
            "arm" => is_arm(asset),
            _ => false,
        })
        .filter(|asset| {
            [
                ".sig",
                ".deb",
                ".rpm",
                ".dmg",
                ".apk",
                ".msi",
                ".sbom",
                ".checksum",
                ".sha256sum",
            ]
            .iter()
            .all(|ext| !asset.name.ends_with(ext))
        })
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

fn is_x86_64(asset: &Asset) -> bool {
    asset.name.contains("amd64")
        || asset.name.contains("x86_64")
        || asset.name.contains("x64")
        || asset.name.contains("x86-64")
}

fn is_aarch64(asset: &Asset) -> bool {
    asset.name.contains("arm64") || asset.name.contains("aarch64")
}

fn is_x86(asset: &Asset) -> bool {
    !is_x86_64(asset) && (asset.name.contains("386") || asset.name.contains("x86") || asset.name.contains("i686"))
}

fn is_arm(asset: &Asset) -> bool {
    !is_aarch64(asset) && asset.name.contains("arm")
}
