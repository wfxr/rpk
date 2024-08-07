pub mod installer;

use std::collections::BTreeMap;

use anyhow::{Context as _, Result};
use futures::{stream, StreamExt, TryStreamExt};
use installer::install_package;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    config::{Config, LockedConfig, LockedPackage, Package},
    context::Context,
    provider::{github::Github, Provider},
    util::fs_ext::load_toml,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SyncResult {
    Updated,
    Checked,
}

pub async fn sync_package(
    ctx: &Context,
    pkg: &Package,
    lpkg: Option<&LockedPackage>,
) -> Result<(LockedPackage, SyncResult)> {
    let provider = Github::new()?;
    let new_lpkg = provider.download(ctx, pkg).await?;

    install_package(ctx, &new_lpkg).await?;

    let res = match lpkg {
        Some(old_pkg) if old_pkg != &new_lpkg => {
            ctx.log_status("Updated", &format!("{}@{}", pkg.name, new_lpkg.version));
            SyncResult::Updated
        }
        _ => {
            ctx.log_status("Checked", &format!("{}@{}", pkg.name, new_lpkg.version));
            SyncResult::Checked
        }
    };
    Ok((new_lpkg, res))
}

pub async fn restore_package(ctx: &Context, lpkg: &LockedPackage) -> Result<()> {
    let provider = Github::new()?;

    provider.download_locked(ctx, lpkg).await?;

    install_package(ctx, lpkg).await?;
    ctx.log_status("Checked", &format!("{}@{}", lpkg.name, lpkg.version));

    Ok(())
}

/// Install all necessary packages, and returns a [`LockedConfig`].
pub async fn sync_packages(ctx: &Context, config: Config) -> Result<LockedConfig> {
    let mut locked = BTreeMap::new();
    for (name, pkg) in config.pkgs.iter() {
        let (lpkg, _) = sync_package(ctx, pkg, None).await?;
        locked.insert(name.clone(), lpkg);
    }

    Ok(LockedConfig::new(ctx.clone(), locked))
}

/// Restore packages according to the given [`LockedConfig`].
pub async fn restore_packages(lcfg: LockedConfig) -> Result<()> {
    let _: Vec<()> = stream::iter(lcfg.pkgs.values())
        .then(|pkg| restore_package(&lcfg.ctx, pkg))
        .try_collect()
        .await?;

    Ok(())
}

impl LockedConfig {
    pub async fn load(ctx: &Context) -> Result<Self> {
        let mut lcfg: LockedConfig = load_toml(&ctx.lock_file)
            .await
            .with_context(|| format!("failed to load {}", ctx.lock_file.display()))?;
        lcfg.ctx = ctx.clone();
        Ok(lcfg)
    }

    /// Write this `LockedConfig` to the given path.
    pub async fn save(&self) -> Result<()> {
        let buf = toml::to_string_pretty(self).context("failed to serialize `LockedConfig`")?;
        fs::write(&self.ctx.lock_file, buf)
            .await
            .with_context(|| format!("failed to save {}", self.ctx.lock_file.display()))
    }

    /// Update a package in the configuration. If the package does not exist, add it.
    pub fn upsert(&mut self, lpkg: LockedPackage) {
        self.pkgs.insert(lpkg.name.clone(), lpkg);
    }
}
