#![feature(anonymous_lifetime_in_impl_trait)]
pub mod cli;
pub mod config;
pub mod context;
pub mod lock;
pub mod provider;
pub mod util;

use std::process;

use anyhow::Context;
use cli::Opt;
use context::log_error;
use lock::{restore_package, restore_packages, sync_packages, LockedConfig};
use tracing_subscriber::EnvFilter;
use util::fs_ext::load_toml;

async fn try_main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().with_file(true).with_line_number(true))
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let Opt { command, ctx } = cli::from_args().await;

    match command {
        cli::Command::Init => todo!(),
        cli::Command::Add(_pkg) => todo!(),
        cli::Command::Sync => {
            let cfg = load_toml(&ctx.config_file)
                .await
                .with_context(|| format!("failed to load {}", ctx.config_file.display()))?;
            ctx.log_header("Loaded", ctx.config_file.as_path());

            let lcfg = sync_packages(&ctx, cfg).await?;

            lcfg.save()
                .await
                .with_context(|| format!("failed to save {}", ctx.lock_file.display()))?;
            ctx.log_header("Locked", ctx.lock_file.as_path());
        }
        cli::Command::Restore { package } => {
            let lcfg: LockedConfig = load_toml(&ctx.lock_file)
                .await
                .with_context(|| format!("failed to load {}", ctx.lock_file.display()))?;
            ctx.log_header("Loaded", ctx.lock_file.as_path());

            match package {
                Some(pkg) => {
                    let lpkg = lcfg
                        .pkgs
                        .into_iter()
                        .find(|lpkg| lpkg.base.name == pkg)
                        .with_context(|| format!("package {} not found", pkg))?;
                    restore_package(&ctx, lpkg).await?;
                }
                None => restore_packages(&ctx, lcfg).await?,
            }
        }
        cli::Command::Update { package: _ } => todo!(),
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
