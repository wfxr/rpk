use std::{
    cmp::Reverse,
    env::{
        self,
        consts::{ARCH, OS},
    },
};

use anyhow::{anyhow, Context as _, Result};
use models::{Asset, Release, RepoSearchResult, Repository};
use tracing::{debug, trace, warn};
use ureq::Agent;
use url::Url;

use crate::{
    config::{LockedPackage, Package, Source},
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
        let token = env::var("GITHUB_TOKEN").or_else(|_| env::var("RPK_GITHUB_TOKEN")).ok();

        let agent = ureq::AgentBuilder::new()
            .user_agent("rpk")
            .middleware(BearerAuthMiddleware(token))
            .build();

        Ok(Github { client: agent, ctx })
    }

    pub fn search_repo(&self, query: &str, size: impl Into<u8>) -> Result<Vec<Repository>> {
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

    pub fn get_release(&self, repo: &str, version: Option<&str>) -> Result<Release> {
        match version {
            Some(version) => self
                .client
                .get(&format!("https://api.github.com/repos/{repo}/releases/tags/{version}",))
                .call(),
            None => self
                .client
                .get(&format!("https://api.github.com/repos/{repo}/releases/latest"))
                .call(),
        }
        .context(format!(
            "failed to get release: `{repo}@{version}`",
            version = version.unwrap_or("latest")
        ))?
        .into_json()
        .map_err(Into::into)
    }

    pub fn get_repo(&self, repo: &str) -> Result<Repository> {
        self.client
            .get(&format!("https://api.github.com/repos/{}", repo))
            .call()
            .context(format!("failed to get repo: `{repo}`"))?
            .into_json()
            .map_err(Into::into)
    }

    pub fn parse_repo<'a>(&self, repo: &'a str) -> Result<(&'a str, &'a str)> {
        repo.split_once('/').context(format!("Invalid repo: `{repo}`"))
    }

    pub fn download_asset(&self, name: &str, url: Url) -> Result<()> {
        self.ctx.log_verbose_status("Downloading", &url);
        self.client
            .download(url, self.ctx.cache_dir.join(name))
            .context("failed to download asset")?;
        self.ctx.log_status("Downloaded", name);
        Ok(())
    }
}

impl Provider for Github {
    fn download(&self, ctx: &Context, pkg: &Package) -> Result<LockedPackage> {
        let repo = match &pkg.source {
            Source::Github { repo } => repo,
        };

        let release = self.get_release(repo, pkg.version.as_deref())?;
        ctx.log_verbose_status("Fetched", format!("{repo}@{version}", version = release.tag_name));

        let asset = filter_assets(&release)?;
        let asset = asset.ok_or_else(|| anyhow!("No matching asset found for {repo}@{}", release.tag_name))?;
        ctx.log_verbose_status("Filtered", &asset.name);

        let path = ctx.cache_dir.join(&asset.name);

        // skip download if the asset already exists
        if path.exists() {
            ctx.log_verbose_status("Skipped", format!("Asset already exists: {}", asset.name));
        } else {
            self.download_asset(&asset.name, asset.browser_download_url.clone())?;
        }

        // get description from the release if not provided
        let desc = match &pkg.desc {
            Some(desc) => desc.clone().into(),
            None => self.get_repo(repo).ok().and_then(|repo| repo.description),
        };

        Ok(LockedPackage {
            name:         pkg.name.clone(),
            version:      release.tag_name.clone(),
            source:       pkg.source.clone(),
            desc:         desc.map(|desc| desc.trim().to_string()),
            filename:     asset.name.clone(),
            download_url: asset.browser_download_url.clone().into(),
        })
    }

    fn download_locked(&self, ctx: &Context, lpkg: &LockedPackage) -> Result<()> {
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
                let release = self.get_release(repo, Some(version))?;
                ctx.log_verbose_status("Fetched", format!("{owner}/{repo}@{version}"));
                let asset = release
                    .assets
                    .iter()
                    .find(|asset| asset.name == lpkg.filename)
                    .ok_or_else(|| anyhow::anyhow!("Asset not found"))?;
                asset.browser_download_url.clone()
            }
        };

        self.download_asset(&lpkg.filename, download_url)?;

        Ok(())
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
            trace!("before filter: {asset}", asset = asset.name);
        })
        .filter(|asset| match OS {
            "linux" => is_linux(&asset.name),
            "macos" => is_macos(&asset.name),
            _ => {
                warn!("unsupported OS: {OS}", OS = OS);
                false
            }
        })
        .filter(|asset| match ARCH {
            "x86_64" => is_x86_64(&asset.name),
            "x86" => is_x86(&asset.name),
            // apple silicon macs can run x86_64 binaries
            "aarch64" => is_aarch64(&asset.name) || is_macos(&asset.name) && is_x86_64(&asset.name),
            "arm" => is_arm(&asset.name),
            _ => {
                warn!("unsupported ARCH: {ARCH}", ARCH = ARCH);
                false
            }
        })
        .filter(|asset| {
            ends_with_any!(
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
        .collect::<Vec<_>>();

    assets.sort_by_key(|asset| Reverse(priority(asset)));

    match &assets[..] {
        [] => Ok(None),
        [asset] => Ok(Some(asset)),
        [asset, ..] => {
            warn!(
                "{} assets found, the first one will be used: {:?}",
                assets.len(),
                assets.iter().map(|asset| &asset.name).collect::<Vec<_>>()
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
