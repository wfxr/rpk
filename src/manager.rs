use anyhow::Result;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    config::{Config, LockedConfig, LockedPackage, Package},
    context::Context,
    installer::install_package,
    provider::{Github, Provider},
};

pub fn sync_package(
    ctx: &Context,
    pkg: &Package,
    lpkg: Option<&LockedPackage>,
    update: bool,
) -> Result<LockedPackage> {
    match lpkg {
        // If the package is already installed and the version matches, do nothing.
        Some(lpkg) if pkg.source == lpkg.source || !update => {
            ctx.log_status("Checked", match lpkg.version() {
                Some(v) => format!("{}@{}", pkg.name, v),
                None => pkg.name.clone(),
            });
            Ok(lpkg.clone())
        }
        _ => {
            let provider = Github::new(ctx.clone())?;
            let new_lpkg = provider.lock(pkg)?;

            provider.download(&new_lpkg)?;

            install_package(ctx, &new_lpkg)?;

            let new_ver = new_lpkg.version().unwrap_or("latest");
            match lpkg {
                Some(old_lpkg) if old_lpkg != &new_lpkg => {
                    let old_ver = old_lpkg.version();

                    ctx.log_status("Updated", match old_ver {
                        Some(old_ver) => format!("{}@{} => {}", pkg.name, old_ver, new_ver),
                        None => format!("{} => {}", pkg.name, new_ver),
                    });
                }
                _ => {
                    ctx.log_status("Checked", format!("{}@{}", pkg.name, new_ver));
                }
            };
            Ok(new_lpkg)
        }
    }
}

pub fn restore_package(ctx: &Context, lpkg: &LockedPackage) -> Result<()> {
    let provider = Github::new(ctx.clone())?;

    provider.download(lpkg)?;

    install_package(ctx, lpkg)?;
    ctx.log_status(
        "Checked",
        format!("{}@{}", lpkg.name, lpkg.version().unwrap_or("latest")),
    );

    Ok(())
}

/// Install all necessary packages, and returns a [`LockedConfig`].
pub fn sync_packages(
    ctx: &Context,
    cfg: &Config,
    lcfg: &mut LockedConfig,
    update: bool,
) -> Result<()> {
    let new_lpkgs: Vec<_> = cfg
        .pkgs
        .par_iter()
        .map(|(name, pkg)| {
            let old_lpkg = lcfg.pkgs.get(name);
            sync_package(ctx, pkg, old_lpkg, update)
        })
        .collect::<Result<_>>()?;

    lcfg.pkgs = new_lpkgs
        .into_iter()
        .map(|lpkg| (lpkg.name.clone(), lpkg))
        .collect();
    Ok(())
}

/// Restore packages according to the given [`LockedConfig`].
pub fn restore_packages(lcfg: LockedConfig) -> Result<()> {
    lcfg.pkgs.into_par_iter().for_each(|(_, pkg)| {
        restore_package(&lcfg.ctx, &pkg).unwrap();
    });

    Ok(())
}
