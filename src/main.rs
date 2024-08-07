#![feature(anonymous_lifetime_in_impl_trait)]
pub mod cli;
pub mod commands;
pub mod config;
pub mod context;
pub mod manager;
pub mod provider;
pub mod util;

use std::process;

use cli::Opt;
use context::{log_error, Context};
use tracing_subscriber::EnvFilter;

async fn try_main(command: cli::Command, ctx: Context) -> anyhow::Result<()> {
    match command {
        cli::Command::Init => {
            todo!();
        }
        cli::Command::Add(pkg) => {
            commands::add(&ctx, pkg).await?;
        }
        cli::Command::Sync => {
            commands::sync(&ctx).await?;
        }
        cli::Command::Restore { package } => {
            commands::restore(&ctx, package).await?;
        }
        cli::Command::Update { package } => {
            commands::update(&ctx, package).await?;
        }
        cli::Command::Search { query, top } => {
            commands::search(query, top, ctx).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().with_file(true).with_line_number(true))
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let Opt { command, ctx } = cli::from_args().await;

    if let Err(e) = try_main(command, ctx).await {
        log_error(true, &e);
        process::exit(1);
    }
}
