pub mod installer;

use std::fs;

use anyhow::{anyhow, bail, Error, Result};
use futures::{stream, StreamExt, TryStreamExt};
use installer::install_package;
use serde::{Deserialize, Serialize};
use sha256::try_async_digest;

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
    pub pkg: Package,

    pub filename: String,
    pub checksum: String,
}

// Install a package.
pub async fn lock_package(ctx: &Context, provider: impl Provider, pkg: Package) -> Result<LockedPackage> {
    let path = provider.download(ctx.clone(), &pkg).await?;

    install_package(ctx, &pkg, &path).await?;

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("missing filename"))?
        .to_owned();

    let checksum = try_async_digest(&path).await?;
    ctx.log_status("Checked", &format!("{}@{}", pkg.name, pkg.version));

    Ok(LockedPackage { pkg, filename, checksum })
}

pub async fn check_package(ctx: &Context, provider: impl Provider, pkg: LockedPackage) -> Result<()> {
    let path = provider.download_locked(ctx.clone(), &pkg).await?;

    let locked_checksum = pkg.checksum;

    let checksum = try_async_digest(&path).await?;
    if checksum != locked_checksum {
        ctx.log_status("Mismatch", &format!("{}@{}", pkg.pkg.name, pkg.pkg.version));
        bail!("checksum mismatch");
    }

    install_package(ctx, &pkg.pkg, &path).await?;
    ctx.log_status("Checked", &format!("{}@{}", pkg.pkg.name, pkg.pkg.version));

    Ok(())
}

/// Installs all necessary packages, and returns a [`LockedConfig`].
pub async fn lock_packages(ctx: &Context, config: Config) -> Result<LockedConfig> {
    let provider = Github::new()?;

    let locked = stream::iter(config.pkgs.into_iter())
        .then(|pkg| lock_package(ctx, &provider, pkg))
        .try_collect()
        .await?;

    Ok(LockedConfig { ctx: ctx.clone(), pkgs: locked, errors: Vec::new() })
}

/// Installs all necessary packages according to the given [`LockedConfig`].
pub async fn check_packages(ctx: &Context, config: LockedConfig) -> Result<()> {
    let provider = Github::new()?;

    // TODO: refactor this
    let _: Vec<()> = stream::iter(config.pkgs.into_iter())
        .then(|pkg| check_package(ctx, &provider, pkg))
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
