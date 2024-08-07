#![deny(missing_docs)]

use std::path::PathBuf;

use clap::Parser;

use crate::{cli::color_choice::ColorChoice, util::build};

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
pub struct RawOpt {
    /// Suppress any informational output.
    #[clap(long, short)]
    pub quiet: bool,

    /// Use verbose output.
    #[clap(long, short)]
    pub verbose: bool,

    /// Output coloring: always, auto, or never.
    #[clap(long, value_name = "WHEN", default_value_t)]
    pub color: ColorChoice,

    /// The configuration directory.
    #[clap(long, value_name = "PATH", env = "SHELDON_CONFIG_DIR")]
    pub config_dir: Option<PathBuf>,

    /// The data directory.
    #[clap(long, value_name = "PATH", env = "SHELDON_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// The cache directory.
    #[clap(long, value_name = "PATH", env = "SHELDON_CACHE_DIR")]
    pub cache_dir: Option<PathBuf>,

    /// The directory installed binaries linked to.
    #[clap(long, value_name = "PATH", env = "SHELDON_CACHE_DIR")]
    pub bin_dir: Option<PathBuf>,

    /// The subcommand to run.
    #[clap(subcommand)]
    pub command: RawCommand,
}

#[derive(Debug, PartialEq, Eq, Parser)]
pub enum RawCommand {
    /// Initialize a new config file.
    Init,

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

    /// Restore all packages to the state in the lockfile. For a single package,
    /// restore it to the state in the lockfile.
    Restore {
        /// The packages to restore.
        #[clap(value_name = "PKG")]
        package: Option<String>,
    },

    /// Check and install any missing packages, re-generating the lock file.
    Sync,

    /// Update all packages and re-generate the lock file. For a single package,
    /// update it and re-generate the lock file.
    Update {
        /// The packages to update.
        #[clap(value_name = "PKG")]
        package: Option<String>,
    },

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
