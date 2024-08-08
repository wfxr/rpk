#![feature(anonymous_lifetime_in_impl_trait)]
pub mod cli;
pub mod commands;
pub mod config;
pub mod context;
pub mod manager;
pub mod provider;
pub mod util;

use std::process;

use anyhow::{anyhow, bail, Context as _};
use clap::{CommandFactory as _, Parser as _};
use clap_complete::{generate, generate_to};
use cli::{Opt, SubCommand, ENV_BIN_DIR, ENV_CACHE_DIR, ENV_CONFIG_DIR, ENV_DATA_DIR};
use config::{Package, Source};
use context::{log_error, Context, Output, Verbosity};
use tracing_subscriber::EnvFilter;
use util::{
    build::{self, CRATE_NAME},
    fs_ext::mkdir_p,
};

async fn try_main() -> anyhow::Result<()> {
    let Opt {
        quiet,
        verbose,
        color,
        bin_dir,
        data_dir,
        cache_dir,
        config_dir,
        command,
    } = Opt::parse();

    let verbosity = if quiet {
        Verbosity::Quiet
    } else if verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };

    let output = Output { verbosity, no_color: !color.is_color() };

    let xdg_dirs = xdg::BaseDirectories::with_prefix(CRATE_NAME)?;
    let home = home::home_dir().ok_or_else(|| anyhow!("failed to determine the current user's home directory"))?;

    let config_dir = config_dir.unwrap_or_else(|| xdg_dirs.get_config_home());
    mkdir_p(&config_dir).await.context("failed to create config dir")?;

    let cache_dir = cache_dir.unwrap_or_else(|| xdg_dirs.get_cache_home());
    mkdir_p(&cache_dir).await.context("failed to create cache dir")?;

    let data_dir = data_dir.unwrap_or_else(|| xdg_dirs.get_data_home().join("packages"));
    mkdir_p(&data_dir).await.context("failed to create data dir")?;

    let bin_dir = bin_dir.unwrap_or_else(|| xdg_dirs.get_data_home().join("bin"));
    mkdir_p(&bin_dir).await.context("failed to create binary dir")?;

    let config_file = config_dir.join("packages.toml");
    let lock_file = config_dir.join("packages.lock");

    let version = build::CRATE_RELEASE.to_string();
    let ctx = Context {
        version,
        config_file,
        config_dir,
        cache_dir,
        data_dir,
        bin_dir,
        home,
        lock_file,
        output,
    };

    match command {
        SubCommand::Env => {
            println!("{}='{}'", ENV_CONFIG_DIR, ctx.config_dir.display());
            println!("{}='{}'", ENV_CACHE_DIR, ctx.cache_dir.display());
            println!("{}='{}'", ENV_DATA_DIR, ctx.data_dir.display());
            println!("{}='{}'", ENV_BIN_DIR, ctx.bin_dir.display());
        }
        SubCommand::Add { name, repo, version, desc } => {
            let name = match name {
                Some(name) => name,
                None => match repo.split_once('/') {
                    Some((_owner, repo)) => repo.to_owned(),
                    None => bail!("invalid repo format: `{}`", repo),
                },
            };
            let source = Source::Github { repo };

            let pkg = Package { name, source, version, desc };

            commands::add(&ctx, pkg).await?;
        }
        SubCommand::Sync => {
            commands::sync(&ctx).await?;
        }
        SubCommand::Update { package } => {
            commands::update(&ctx, package).await?;
        }
        SubCommand::Version => {
            println!("{} {}", build::CRATE_NAME, build::CRATE_VERBOSE_VERSION);
        }
        SubCommand::Restore { package } => {
            commands::restore(&ctx, package).await?;
        }
        SubCommand::Search { query, top } => {
            commands::search(query, top, ctx).await?;
        }
        SubCommand::Completions { shell, dir } => {
            let cmd = &mut Opt::command();
            match dir {
                Some(dir) => {
                    let path = generate_to(shell, cmd, cmd.get_name().to_string(), dir)?;
                    ctx.log_status("Generated", path.as_path());
                }
                None => generate(shell, cmd, cmd.get_name().to_string(), &mut std::io::stdout()),
            }
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

    if let Err(e) = try_main().await {
        log_error(true, &e);
        process::exit(1);
    }
}
