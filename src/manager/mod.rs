pub mod installer;

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
    update: bool,
) -> Result<(LockedPackage, SyncResult)> {
    match (&pkg.version, lpkg) {
        // If the package is already installed and the version matches, do nothing.
        (Some(version), Some(lpkg)) if version == &lpkg.version => {
            ctx.log_status("Checked", &format!("{}@{}", pkg.name, lpkg.version));
            Ok((lpkg.clone(), SyncResult::Checked))
        }
        (None, Some(lpkg)) if !update => {
            ctx.log_status("Checked", &format!("{}@{}", pkg.name, lpkg.version));
            Ok((lpkg.clone(), SyncResult::Checked))
        }
        _ => {
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
    }
}

pub async fn restore_package(ctx: &Context, lpkg: &LockedPackage) -> Result<()> {
    let provider = Github::new()?;

    provider.download_locked(ctx, lpkg).await?;

    install_package(ctx, lpkg).await?;
    ctx.log_status("Checked", &format!("{}@{}", lpkg.name, lpkg.version));

    Ok(())
}

/// Install all necessary packages, and returns a [`LockedConfig`].
pub async fn sync_packages(ctx: &Context, cfg: &Config, lcfg: &mut LockedConfig) -> Result<()> {
    for (name, pkg) in cfg.pkgs.iter() {
        let old_lpkg = lcfg.pkgs.get(name);
        let (new_lpkg, _) = sync_package(ctx, pkg, old_lpkg, false).await?;
        lcfg.upsert(new_lpkg);
    }

    Ok(())
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

        // Set the package names for convenience.
        for (name, lpkg) in lcfg.pkgs.iter_mut() {
            lpkg.name = name.clone();
        }
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
