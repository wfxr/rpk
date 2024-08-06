pub mod installer;

use std::fs;

use anyhow::{Error, Result};
use futures::{stream, StreamExt, TryStreamExt};
use installer::install_package;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    config::{Config, Package},
    context::Context,
    provider::{github::Github, Provider},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct LockedConfig {
    #[serde(flatten)]
    ctx: Context,

    pub pkgs: Vec<LockedPackage>,

    /// Any errors that occurred while generating this `LockedConfig`.
    #[serde(skip)]
    pub errors: Vec<Error>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "source")]
#[serde(rename_all = "lowercase")]
pub struct LockedPackage {
    #[serde(flatten)]
    pub base: Package,

    pub filename:     String,
    pub download_url: Option<Url>,
}

// Install a package.
pub async fn sync_package(ctx: &Context, provider: impl Provider, pkg: Package) -> Result<LockedPackage> {
    let locked_package = provider.download(ctx, &pkg).await?;

    install_package(ctx, &locked_package).await?;
    ctx.log_status("Checked", &format!("{}@{}", pkg.name, pkg.version));

    Ok(locked_package)
}

pub async fn restore_package(ctx: &Context, provider: impl Provider, lpkg: LockedPackage) -> Result<()> {
    provider.download_locked(ctx, &lpkg).await?;

    install_package(ctx, &lpkg).await?;
    ctx.log_status("Checked", &format!("{}@{}", lpkg.base.name, lpkg.base.version));

    Ok(())
}

/// Install all necessary packages, and returns a [`LockedConfig`].
pub async fn sync_packages(ctx: &Context, config: Config) -> Result<LockedConfig> {
    let provider = Github::new()?;

    let locked = stream::iter(config.pkgs.into_iter())
        .then(|pkg| sync_package(ctx, &provider, pkg))
        .try_collect()
        .await?;

    Ok(LockedConfig { ctx: ctx.clone(), pkgs: locked, errors: Vec::new() })
}

/// Restore packages according to the given [`LockedConfig`].
pub async fn restore_packages(ctx: &Context, config: LockedConfig) -> Result<()> {
    let provider = Github::new()?;

    // TODO: refactor this
    let _: Vec<()> = stream::iter(config.pkgs.into_iter())
        .then(|pkg| restore_package(ctx, &provider, pkg))
        .try_collect()
        .await?;

    Ok(())
}

impl LockedConfig {
    /// Write this `LockedConfig` to the given path.
    pub fn save(&self) -> Result<()> {
        let buf = toml::to_string_pretty(self)?;
        fs::write(&self.ctx.lock_file, buf)?;
        Ok(())
    }
}
