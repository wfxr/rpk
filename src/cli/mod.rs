//! Command line interface.

mod color_choice;
mod raw;

use std::process;

use anyhow::{anyhow, bail, Context as ResultExt, Result};
use clap::Parser;
use tracing::debug;

use crate::{
    cli::raw::{RawCommand, RawOpt},
    config::{Package, Source},
    context::{log_error, Context, Output, Verbosity},
    util::{
        build::{self, CRATE_NAME},
        fs_ext::mkdir_p,
    },
};

/// Parse the command line arguments.
///
/// In the event of failure it will print the error message and quit the program
/// without returning.
pub async fn from_args() -> Opt {
    match Opt::from_raw_opt(RawOpt::parse()).await {
        Ok(opt) => {
            debug!("parsed options: {:?}", opt);
            opt
        }
        Err(err) => {
            log_error(true, &err);
            process::exit(1);
        }
    }
}

/// Resolved command line options with defaults set.
#[derive(Debug)]
pub struct Opt {
    /// Global context for use across the entire program.
    pub ctx:     Context,
    /// The subcommand.
    pub command: Command,
}

/// The resolved command.
#[derive(Debug)]
pub enum Command {
    /// Initialize a new config file.
    Init,

    /// Add a new plugin to the config file.
    Add(Package),

    /// Check and install any missing packages.
    Check {
        /// require that config.lock is up-to-date.
        locked: bool,
    },

    /// Update the packages and regenerate the lock file.
    Update {
        /// The packages to update.
        package: Option<String>,
    },
}

impl Opt {
    async fn from_raw_opt(raw_opt: RawOpt) -> Result<Self> {
        let RawOpt {
            quiet,
            verbose,
            color,
            bin_dir,
            data_dir,
            cache_dir,
            config_dir,
            command,
        } = raw_opt;

        let command = match command {
            RawCommand::Init => Command::Init,
            RawCommand::Add { name, repo, version, desc } => {
                let name = match name {
                    Some(name) => name,
                    None => match repo.split_once('/') {
                        Some((_owner, repo)) => repo.to_owned(),
                        None => bail!("invalid repo format: `{}`", repo),
                    },
                };
                let source = Source::Github { repo };

                Command::Add(Package { name, source, version, desc })
            }
            RawCommand::Check { locked } => Command::Check { locked },
            RawCommand::Update { package } => Command::Update { package },
            RawCommand::Version => {
                println!("{} {}", build::CRATE_NAME, build::CRATE_VERBOSE_VERSION);
                process::exit(0);
            }
        };

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

        Ok(Self { ctx, command })
    }
}
