use std::{
    env,
    io::{self, IsTerminal as _},
    path::PathBuf,
};

use clap::{ColorChoice, Parser};
use clap_complete::Shell;
use url::Url;

use crate::{
    context::{Output, Verbosity},
    util,
};

pub const ENV_CONFIG_DIR: &str = "RPK_CONFIG_DIR";
pub const ENV_DATA_DIR: &str = "RPK_DATA_DIR";
pub const ENV_CACHE_DIR: &str = "RPK_CACHE_DIR";
pub const ENV_BIN_DIR: &str = "RPK_BIN_DIR";

/// Resolved command line options.
#[derive(Debug, PartialEq, Eq, Parser)]
#[clap(author, about)]
#[clap(version = util::CRATE_VERSION)]
#[clap(long_version = util::CRATE_LONG_VERSION)]
pub struct Opt {
    /// Suppress any informational output.
    #[clap(long, short)]
    pub quiet: bool,

    /// Use verbose output.
    #[clap(long, short)]
    pub verbose: bool,

    /// This flag controls when to use colors.
    #[clap(long, value_enum, value_name = "WHEN", default_value_t = ColorChoice::Auto, ignore_case = true)]
    pub color: ColorChoice,

    /// The configuration directory.
    #[clap(long, value_name = "PATH", env = ENV_CONFIG_DIR)]
    pub config_dir: Option<PathBuf>,

    /// The directory to store package data.
    #[clap(long, value_name = "PATH", env = ENV_DATA_DIR)]
    pub data_dir: Option<PathBuf>,

    /// The directory to store downloaded packages.
    #[clap(long, value_name = "PATH", env = ENV_CACHE_DIR)]
    pub cache_dir: Option<PathBuf>,

    /// The directory installed binaries linked to.
    #[clap(long, value_name = "PATH", env = ENV_BIN_DIR)]
    pub bin_dir: Option<PathBuf>,

    /// The subcommand to run.
    #[clap(subcommand)]
    pub command: SubCommand,
}

/// The resolved sub command.
#[derive(Debug, PartialEq, Eq, Parser)]
pub enum SubCommand {
    /// Initialize a configuration file.
    Init {
        /// The config file URL to initialize from.
        #[clap(short, long, value_name = "URL")]
        from: Option<Url>,
    },

    /// List all installed packages.
    #[clap(visible_aliases = ["l", "ls"])]
    List,

    /// Install any missing packages, re-generating the lock file.
    #[clap(visible_alias = "s")]
    Sync,

    /// Add a new plugin to the config file.
    #[clap(visible_alias = "a")]
    Add {
        /// The github repository hosting the package
        ///
        /// Example: `sharkdp/fd`
        #[clap(value_name = "REPO")]
        repo: String,

        /// A unique name for the package.
        #[clap(long, value_name = "NAME")]
        name: Option<String>,

        /// The version of the package.
        #[clap(long, value_name = "VERSION")]
        version: Option<String>,

        /// A description of the package.
        #[clap(long, value_name = "DESC", long)]
        desc: Option<String>,
    },

    /// Restore packages to the state in the lockfile.
    #[clap(visible_alias = "r")]
    Restore {
        /// The packages to restore.
        #[clap(value_name = "PKG")]
        package: Option<String>,
    },

    /// Update packages and re-generate the lock file.
    #[clap(visible_alias = "u")]
    Update {
        /// The packages to update.
        #[clap(value_name = "PKG")]
        package: Option<String>,
    },

    /// Find packages matching the given query.
    #[clap(visible_aliases = ["f", "fd"])]
    Find {
        /// The query to search for.
        #[clap(value_name = "QUERY")]
        query: String,

        /// The number of results to display.
        #[clap(long, value_name = "NUM", default_value = "10")]
        top: u8,
    },

    /// print environment information
    Env,

    /// Generate completions for the given shell.
    Completions {
        /// The shell to generate completions for.
        #[clap(value_name = "SHELL", value_enum)]
        shell: Shell,

        /// The directory to write the completions to.
        ///
        /// Defaults output to stdout.
        #[clap(short, long, value_name = "DIR")]
        dir: Option<PathBuf>,
    },

    /// Prints detailed version information.
    Version,
}

impl Opt {
    pub fn color_enabled(&self) -> bool {
        let enabled = match self.color {
            ColorChoice::Always => true,
            ColorChoice::Auto => io::stderr().is_terminal() && env::var("NO_COLOR").is_err(),
            ColorChoice::Never => false,
        };
        match enabled {
            true => env::remove_var("NO_COLOR"),
            false => env::set_var("NO_COLOR", "1"),
        }
        enabled
    }

    pub fn output_opt(&self) -> Output {
        Output {
            verbosity: if self.quiet {
                Verbosity::Quiet
            } else if self.verbose {
                Verbosity::Verbose
            } else {
                Verbosity::Normal
            },
            no_color:  !self.color_enabled(),
        }
    }
}
