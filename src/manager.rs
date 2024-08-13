use anyhow::Result;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    config::{Config, LockedConfig, LockedPackage, Package},
    context::Context,
    installer::install_package,
    provider::{Github, Provider},
};

pub fn sync_package(ctx: &Context, pkg: &Package, lpkg: Option<&LockedPackage>, update: bool) -> Result<LockedPackage> {
    match (&pkg.version, lpkg) {
        // If the package is already installed and the version matches, do nothing.
        (Some(version), Some(lpkg)) if version == &lpkg.version => {
            ctx.log_status("Checked", format!("{}@{}", pkg.name, lpkg.version));
            Ok(lpkg.clone())
        }
        (None, Some(lpkg)) if !update => {
            ctx.log_status("Checked", format!("{}@{}", pkg.name, lpkg.version));
            Ok(lpkg.clone())
        }
        _ => {
            let provider = Github::new(ctx.clone())?;
            let new = provider.download(ctx, pkg)?;

            install_package(ctx, &new)?;

            match lpkg {
                Some(old) if old != &new => {
                    ctx.log_status("Updated", format!("{}@{} => {}", pkg.name, old.version, new.version));
                }
                _ => {
                    ctx.log_status("Checked", format!("{}@{}", pkg.name, new.version));
                }
            };
            Ok(new)
        }
    }
}

pub fn restore_package(ctx: &Context, lpkg: &LockedPackage) -> Result<()> {
    let provider = Github::new(ctx.clone())?;

    provider.download_locked(ctx, lpkg)?;

    install_package(ctx, lpkg)?;
    ctx.log_status("Checked", format!("{}@{}", lpkg.name, lpkg.version));

    Ok(())
}

/// Install all necessary packages, and returns a [`LockedConfig`].
pub fn sync_packages(ctx: &Context, cfg: &Config, lcfg: &mut LockedConfig) -> Result<()> {
    let new_lpkgs: Vec<_> = cfg
        .pkgs
        .par_iter()
        .map(|(name, pkg)| {
            let old_lpkg = lcfg.pkgs.get(name);
            sync_package(ctx, pkg, old_lpkg, false)
        })
        .collect::<Result<_>>()?;

    for lpkg in new_lpkgs {
        lcfg.upsert(lpkg);
    }

    Ok(())
}

/// Restore packages according to the given [`LockedConfig`].
pub fn restore_packages(lcfg: LockedConfig) -> Result<()> {
    lcfg.pkgs.into_par_iter().for_each(|(_, pkg)| {
        restore_package(&lcfg.ctx, &pkg).unwrap();
    });

    Ok(())
}
