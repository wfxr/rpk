use std::{
    cmp::Reverse,
    env::{
        self,
        consts::{ARCH, OS},
    },
    fs,
};

use anyhow::{bail, Context as _, Result};
use models::{Asset, Release, RepoSearchResult};
use tracing::{debug, info, trace, warn};
use ureq::Agent;

use crate::{
    config::{LockedPackage, Package, Repository, Source},
    context::Context,
    util::http::{BearerAuthMiddleware, UreqExt as _},
};

use super::Provider;

pub struct Github {
    client: Agent,

    ctx: Context,
}

impl Github {
    pub fn new(ctx: Context) -> Result<Self> {
        let token = env::var("GITHUB_TOKEN")
            .or_else(|_| env::var("RPK_GITHUB_TOKEN"))
            .ok();

        let agent = ureq::AgentBuilder::new()
            .user_agent("rpk")
            .middleware(BearerAuthMiddleware(token))
            .build();

        Ok(Github { client: agent, ctx })
    }

    pub fn search_repo(&self, query: &str, size: impl Into<u8>) -> Result<Vec<models::Repository>> {
        let res: RepoSearchResult = self
            .client
            .get("https://api.github.com/search/repositories")
            .query("q", query)
            .query("per_page", &size.into().to_string())
            .call()
            .context("failed to search repo")?
            .into_json()?;

        Ok(res.items)
    }

    pub fn get_release(&self, repo: &Repository, version: Option<&str>) -> Result<Release> {
        match version {
            Some(version) => self
                .client
                .get(&format!(
                    "https://api.github.com/repos/{repo}/releases/tags/{version}",
                ))
                .call(),
            None => self
                .client
                .get(&format!(
                    "https://api.github.com/repos/{repo}/releases/latest"
                ))
                .call(),
        }
        .with_context(|| {
            format!(
                "failed to get release: `{repo}@{version}`",
                version = version.unwrap_or("latest")
            )
        })?
        .into_json()
        .map_err(Into::into)
    }

    pub fn get_repo(&self, repo: &Repository) -> Result<models::Repository> {
        self.client
            .get(&format!("https://api.github.com/repos/{}", repo))
            .call()
            .with_context(|| format!("failed to get repo: `{repo}`"))?
            .into_json()
            .map_err(Into::into)
    }

    pub fn parse_repo<'a>(&self, repo: &'a str) -> Result<(&'a str, &'a str)> {
        repo.split_once('/')
            .with_context(|| format!("Invalid repo: `{repo}`"))
    }

    pub fn download_asset(&self, lpkg: &LockedPackage) -> Result<()> {
        // skip download if the asset already exists
        if lpkg.asset_path(&self.ctx).exists() {
            self.ctx.log_status_v(
                "Skipped",
                format!("Asset already exists: {}", lpkg.filename),
            );
            return Ok(());
        }

        self.ctx.log_status_v("Downloading", &lpkg.download_url);
        let cache_dir = self
            .ctx
            .cache_dir
            .join(&lpkg.name)
            .join(&lpkg.version().unwrap_or("latest"));
        fs::create_dir_all(&cache_dir).context("failed to create cache directory")?;
        self.client
            .download(&lpkg.download_url, cache_dir.join(&lpkg.filename))
            .context("failed to download asset")?;
        self.ctx.log_status("Downloaded", &lpkg.filename);
        Ok(())
    }
}

impl Provider for Github {
    fn lock(&self, pkg: &Package) -> Result<LockedPackage> {
        let (repo, tag) = match &pkg.source {
            Source::Release { repo, tag } => (repo, tag.as_deref()),
            _ => bail!("unsupported source: {:?}", pkg.source),
        };

        let release = self.get_release(repo, tag)?;
        let tag = release.tag_name.clone();
        self.ctx.log_status_v("Fetched", format!("{repo}@{tag}"));

        let asset = filter_assets(&release)?
            .with_context(|| format!("No matching asset found for {repo}@{}", tag))?;
        self.ctx.log_status_v("Filtered", &asset.name);

        // get description from the release if not provided
        let desc = if !pkg.desc.is_empty() {
            pkg.desc.trim().to_string()
        } else {
            self.get_repo(repo)?
                .description
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };

        Ok(LockedPackage {
            name: pkg.name.clone(),
            bins: pkg.bins.clone(),
            source: Source::Release { repo: repo.clone(), tag: Some(tag) },
            desc,
            filename: asset.name.clone(),
            download_url: asset.browser_download_url.clone(),
        })
    }

    fn download(&self, lpkg: &LockedPackage) -> Result<()> {
        self.download_asset(lpkg)
    }
}

// check if a string contains any of the patterns (case-insensitive)
macro_rules! contains_any {
    ($s:expr, $($pat:expr),+ $(,)?) => {{
        let s = $s.to_lowercase();
        [$($pat),+].iter().any(|pat| s.contains(pat))

    }};
}

// check if a string ends with any of the patterns (case-insensitive)
macro_rules! ends_with_any {
    ($s:expr, $($pat:expr),+ $(,)?) => {{
        let s = $s.to_lowercase();
        [$($pat),+].iter().any(|pat| s.ends_with(&pat.to_lowercase()))

    }};
}

fn filter_assets(release: &Release) -> anyhow::Result<Option<&Asset>> {
    debug!("OS: {OS}, ARCH: {ARCH}");

    let mut assets = release
        .assets
        .iter()
        .inspect(|asset| {
            debug!("before filter: {asset}", asset = asset.name);
        })
        .filter(|asset| {
            trace!("before OS filter: {asset}", asset = asset.name);
            match OS {
                "linux" => is_linux(&asset.name),
                "macos" => is_macos(&asset.name),
                _ => {
                    warn!("unsupported OS: {OS}", OS = OS);
                    false
                }
            }
        })
        .filter(|asset| {
            trace!("before ARCH filter: {asset}", asset = asset.name);
            match ARCH {
                "x86_64" => is_x86_64(&asset.name),
                "x86" => is_x86(&asset.name),
                "aarch64" =>
                    is_aarch64(&asset.name) || is_macos(&asset.name) && is_x86_64(&asset.name),
                "arm" => is_arm(&asset.name),
                _ => {
                    warn!("unsupported ARCH: {ARCH}", ARCH = ARCH);
                    false
                }
            }
        })
        .filter(|asset| {
            trace!("before SUFFIX filter: {asset}", asset = asset.name);
            !ends_with_any!(
                asset.name,
                ".sig",
                ".deb",
                ".rpm",
                ".dmg",
                ".apk",
                ".msi",
                ".sbom",
                ".checksum",
                ".sha256sum"
            )
        })
        .inspect(|asset| {
            debug!("after filter: {asset}", asset = asset.name);
        })
        .collect::<Vec<_>>();

    assets.sort_by_key(|asset| Reverse(priority(asset)));

    match &assets[..] {
        [] => Ok(None),
        [asset] => Ok(Some(asset)),
        [asset, ..] => {
            info!(
                "{} assets found, the first one will be used: {}",
                assets.len(),
                asset.name
            );
            Ok(Some(asset))
        }
    }
}

fn is_linux(filename: &str) -> bool {
    contains_any!(filename, "linux")
}

fn is_macos(filename: &str) -> bool {
    contains_any!(filename, "apple", "darwin", "osx", "mac")
}

fn is_x86_64(filename: &str) -> bool {
    contains_any!(filename, "amd64", "x86_64", "x64", "x86-64")
}

fn is_aarch64(filename: &str) -> bool {
    contains_any!(filename, "arm64", "aarch64")
}

fn is_x86(filename: &str) -> bool {
    !is_x86_64(filename) && contains_any!(filename, "386", "x86", "i686")
}

fn is_arm(filename: &str) -> bool {
    !is_aarch64(filename) && contains_any!(filename, "arm")
}

fn is_musl(filename: &str) -> bool {
    contains_any!(filename, "musl")
}

fn priority(asset: &Asset) -> u64 {
    let mut priority = 0;

    // choose the musl version if available
    priority <<= 1;
    if is_musl(&asset.name) {
        priority += 1;
    }

    // choose the aarch64 version if available
    priority <<= 1;
    if is_aarch64(&asset.name) {
        priority += 1;
    }

    priority
}

mod models {
    use serde::Deserialize;
    use url::Url;

    #[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
    pub struct RepoSearchResult {
        pub total_count:        u32,
        pub incomplete_results: bool,
        pub items:              Vec<Repository>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
    pub struct Repository {
        pub name:              String,
        pub full_name:         Option<String>,
        pub owner:             Option<Author>,
        pub description:       Option<String>,
        pub fork:              Option<bool>,
        pub homepage:          Option<String>,
        pub language:          Option<String>,
        pub forks_count:       Option<u32>,
        pub stargazers_count:  Option<u32>,
        pub watchers_count:    Option<u32>,
        pub size:              Option<u32>,
        pub default_branch:    Option<String>,
        pub open_issues_count: Option<u32>,
        pub is_template:       Option<bool>,
        pub topics:            Option<Vec<String>>,
        pub has_downloads:     Option<bool>,
        pub archived:          Option<bool>,
        pub disabled:          Option<bool>,
        pub visibility:        Option<String>,
        pub pushed_at:         Option<String>,
        pub created_at:        Option<String>,
        pub updated_at:        Option<String>,
        pub subscribers_count: Option<i64>,
        pub network_count:     Option<i64>,
        pub license:           Option<License>,
        pub parent:            Option<Box<Repository>>,
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize)]
    pub struct License {
        pub key:  String,
        pub name: String,
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize)]
    pub struct Author {
        pub login:      String,
        pub avatar_url: Url,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct Release {
        pub name:             Option<String>,
        pub body:             Option<String>,
        pub tag_name:         String,
        pub target_commitish: String,
        pub tarball_url:      Option<Url>,
        pub zipball_url:      Option<Url>,
        pub draft:            bool,
        pub prerelease:       bool,
        pub created_at:       Option<String>,
        pub published_at:     Option<String>,
        pub assets:           Vec<Asset>,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct Asset {
        pub name:                 String,
        pub url:                  Url,
        pub browser_download_url: Url,
        pub label:                Option<String>,
        pub state:                String,
        pub content_type:         String,
        pub size:                 i64,
        pub download_count:       i64,
        pub created_at:           String,
        pub updated_at:           String,
    }
}
