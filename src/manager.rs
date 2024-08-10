use anyhow::Result;
use futures::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::{
    config::{Config, LockedConfig, LockedPackage, Package},
    context::Context,
    installer::install_package,
    provider::{Github, Provider},
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
            ctx.log_status("Checked", format!("{}@{}", pkg.name, lpkg.version));
            Ok((lpkg.clone(), SyncResult::Checked))
        }
        (None, Some(lpkg)) if !update => {
            ctx.log_status("Checked", format!("{}@{}", pkg.name, lpkg.version));
            Ok((lpkg.clone(), SyncResult::Checked))
        }
        _ => {
            let provider = Github::new(ctx.clone())?;
            let new_lpkg = provider.download(ctx, pkg).await?;

            install_package(ctx, &new_lpkg).await?;

            let res = match lpkg {
                Some(old_lpkg) if old_lpkg != &new_lpkg => {
                    ctx.log_status(
                        "Updated",
                        format!("{}@{} => {}", pkg.name, old_lpkg.version, new_lpkg.version),
                    );
                    SyncResult::Updated
                }
                _ => {
                    ctx.log_status("Checked", format!("{}@{}", pkg.name, new_lpkg.version));
                    SyncResult::Checked
                }
            };
            Ok((new_lpkg, res))
        }
    }
}

pub async fn restore_package(ctx: &Context, lpkg: &LockedPackage) -> Result<()> {
    let provider = Github::new(ctx.clone())?;

    provider.download_locked(ctx, lpkg).await?;

    install_package(ctx, lpkg).await?;
    ctx.log_status("Checked", format!("{}@{}", lpkg.name, lpkg.version));

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
