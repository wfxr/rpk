#![feature(anonymous_lifetime_in_impl_trait)]
pub mod cli;
pub mod config;
pub mod context;
pub mod lock;
pub mod provider;
pub mod util;

use std::process;

use cli::Opt;
use context::log_error;
use lock::{restore_packages, sync_packages};
use tracing_subscriber::EnvFilter;
use util::fs_ext::load_toml;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().with_file(true).with_line_number(true))
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let Opt { command, ctx } = cli::from_args().await;

    match command {
        cli::Command::Init => todo!(),
        cli::Command::Add(_pkg) => todo!(),
        cli::Command::Sync => {
            let config = match load_toml(&ctx.config_file).await {
                Ok(config) => config,
                Err(err) => {
                    let context = format!("failed to load {}", ctx.config_file.display());
                    log_error(true, &err.context(context));
                    process::exit(1);
                }
            };
            ctx.log_header("Loaded", ctx.config_file.as_path());

            let locked_config = sync_packages(&ctx, config).await?;
            locked_config.save()?;
            ctx.log_header("Locked", ctx.lock_file.as_path());
        }
        cli::Command::Restore { package: _ } => {
            let config = match load_toml(&ctx.lock_file).await {
                Ok(config) => config,
                Err(err) => {
                    let context = format!("failed to load {}", ctx.config_file.display());
                    log_error(true, &err.context(context));
                    process::exit(1);
                }
            };
            ctx.log_header("Loaded", ctx.lock_file.as_path());
            restore_packages(&ctx, config).await?;
        }
        cli::Command::Update { package: _ } => todo!(),
    }

    Ok(())
}
