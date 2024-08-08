//! Command line interface.
#![deny(missing_docs)]

mod color;

use std::path::PathBuf;

use clap::Parser;

use crate::{cli::color::ColorChoice, util::build};

/// Resolved command line options.
#[derive(Debug, PartialEq, Eq, Parser)]
#[clap(
    author,
    version = build::CRATE_RELEASE,
    long_version = build::CRATE_LONG_VERSION,
    about,
    long_about = None,
    disable_help_subcommand(true),
    subcommand_required(true),
)]
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
    #[clap(long, value_name = "PATH", env = "SHELDON_CONFIG_DIR")]
    pub config_dir: Option<PathBuf>,

    /// The directory to store package data.
    #[clap(long, value_name = "PATH", env = "SHELDON_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// The directory to store downloaded packages.
    #[clap(long, value_name = "PATH", env = "SHELDON_CACHE_DIR")]
    pub cache_dir: Option<PathBuf>,

    /// The directory installed binaries linked to.
    #[clap(long, value_name = "PATH", env = "SHELDON_CACHE_DIR")]
    pub bin_dir: Option<PathBuf>,

    /// The subcommand to run.
    #[clap(subcommand)]
    pub command: SubCommand,
}

/// The resolved sub command.
#[derive(Debug, PartialEq, Eq, Parser)]
pub enum SubCommand {
    /// Initialize a new config file.
    Init,

    /// install any missing packages, re-generating the lock file.
    Sync,

    /// Add a new plugin to the config file.
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
    Restore {
        /// The packages to restore.
        #[clap(value_name = "PKG")]
        package: Option<String>,
    },

    /// Update packages and re-generate the lock file.
    Update {
        /// The packages to update.
        #[clap(value_name = "PKG")]
        package: Option<String>,
    },

    /// Search for packages on GitHub.
    Search {
        /// The query to search for.
        #[clap(value_name = "QUERY")]
        query: String,

        /// The number of results to display.
        #[clap(long, value_name = "NUM", default_value = "10")]
        top: u8,
    },

    /// Prints detailed version information.
    Version,
}
