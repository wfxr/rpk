#![feature(anonymous_lifetime_in_impl_trait)]
pub mod cli;
pub mod config;
pub mod context;
pub mod manager;
pub mod provider;
pub mod util;

use std::process;

use anyhow::Context;
use cli::Opt;
use config::{Config, EditableConfig, LockedConfig};
use context::log_error;
use manager::{restore_package, restore_packages, sync_package, sync_packages, SyncResult};
use tracing_subscriber::EnvFilter;

async fn try_main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().with_file(true).with_line_number(true))
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let Opt { command, ctx } = cli::from_args().await;

    match command {
        cli::Command::Init => todo!(),
        cli::Command::Add(mut pkg) => {
            let mut ecfg = EditableConfig::load(&ctx).await?;
            ctx.log_header("Loaded", ctx.config_file.as_path());

            let (lpkg, _) = sync_package(&ctx, &pkg, None).await?;
            pkg.desc = lpkg.desc.clone();

            ecfg.upsert(&pkg)?;

            let mut lcfg = LockedConfig::load(&ctx).await?;
            lcfg.upsert(lpkg);

            ecfg.save().await?;
            lcfg.save().await?;
            ctx.log_header("Locked", ctx.lock_file.as_path());
        }
        cli::Command::Sync => {
            let cfg = Config::load(&ctx).await?;
            ctx.log_header("Loaded", ctx.config_file.as_path());

            let lcfg = sync_packages(&ctx, cfg).await?;

            lcfg.save().await?;
            ctx.log_header("Locked", ctx.lock_file.as_path());
        }
        cli::Command::Restore { package } => {
            let lcfg = LockedConfig::load(&ctx).await?;
            ctx.log_header("Loaded", ctx.lock_file.as_path());

            match package {
                Some(pkg) => {
                    let lpkg = lcfg
                        .pkgs
                        .get(&pkg)
                        .with_context(|| format!("package {} not found", pkg))?;
                    restore_package(&ctx, lpkg).await?;
                }
                None => restore_packages(lcfg).await?,
            }
        }
        cli::Command::Update { package } => {
            let cfg = Config::load(&ctx).await?;
            ctx.log_header("Loaded", ctx.config_file.as_path());

            match package {
                Some(package) => {
                    let pkg = cfg
                        .pkgs
                        .values()
                        .find(|pkg| pkg.name == package)
                        .cloned()
                        .with_context(|| format!("package {} not found", package))?;

                    let mut lcfg = LockedConfig::load(&ctx).await?;
                    let old_lpkg = lcfg.pkgs.get(&package);

                    // Sync the package.
                    let (new_lpkg, sync_res) = sync_package(&ctx, &pkg, old_lpkg).await?;

                    // Update the package in the lock file.
                    if sync_res == SyncResult::Updated {
                        lcfg.upsert(new_lpkg);
                        lcfg.save().await?;
                        ctx.log_header("Locked", ctx.lock_file.as_path());
                    }
                }
                None => {
                    let mut lcfg = LockedConfig::load(&ctx).await?;
                    ctx.log_header("Loaded", ctx.lock_file.as_path());

                    let mut updated = false;
                    for pkg in cfg.pkgs.values() {
                        let old_lpkg = lcfg.pkgs.get(&pkg.name);

                        // Sync the package.
                        let (new_lpkg, sync_res) = sync_package(&ctx, pkg, old_lpkg).await?;

                        // Update the package in the lock file.
                        if sync_res == SyncResult::Updated {
                            lcfg.upsert(new_lpkg);
                            updated = true;
                        }
                    }

                    if updated {
                        lcfg.save().await?;
                        ctx.log_header("Locked", ctx.lock_file.as_path());
                    }
                }
            };
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = try_main().await {
        log_error(true, &e);
        process::exit(1);
    }
}
