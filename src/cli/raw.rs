#![deny(missing_docs)]

use std::path::PathBuf;

use clap::Parser;

use crate::{cli::color_choice::ColorChoice, util::build};

const HELP_TEMPLATE: &str = "\
{before-help}{bin} {version}
{author}
{about}

{usage-heading}
{tab}{usage}

{all-args}{after-help}";

#[derive(Debug, PartialEq, Eq, Parser)]
#[clap(
    author,
    version = build::CRATE_RELEASE,
    long_version = build::CRATE_LONG_VERSION,
    about,
    long_about = None,
    help_template = HELP_TEMPLATE,
    disable_help_subcommand(true),
    subcommand_required(true),
)]
pub struct RawOpt {
    /// Suppress any informational output.
    #[clap(long, short)]
    pub quiet: bool,

    /// Suppress any interactive prompts and assume "yes" as the answer.
    #[clap(long)]
    pub non_interactive: bool,

    /// Use verbose output.
    #[clap(long, short)]
    pub verbose: bool,

    /// Output coloring: always, auto, or never.
    #[clap(long, value_name = "WHEN", default_value_t)]
    pub color: ColorChoice,

    /// The configuration directory.
    #[clap(long, value_name = "PATH", env = "SHELDON_CONFIG_DIR")]
    pub config_dir: Option<PathBuf>,

    /// The data directory
    #[clap(long, value_name = "PATH", env = "SHELDON_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// The config file.
    #[clap(long, value_name = "PATH", env = "SHELDON_CONFIG_FILE")]
    pub config_file: Option<PathBuf>,

    /// The profile used for conditional plugins.
    #[clap(long, value_name = "PROFILE", env = "SHELDON_PROFILE")]
    pub profile: Option<String>,

    /// The subcommand to run.
    #[clap(subcommand)]
    pub command: RawCommand,
}

#[derive(Debug, PartialEq, Eq, Parser)]
pub enum RawCommand {
    /// Initialize a new config file.
    Init,

    /// Add a new plugin to the config file.
    Add(Box<Add>),

    /// Install the plugins sources and generate the lock file.
    Lock {
        /// Update all plugin sources.
        #[clap(long)]
        update: bool,

        /// Reinstall all plugin sources.
        #[clap(long, conflicts_with = "update")]
        reinstall: bool,
    },

    /// Generate and print out the script.
    Source {
        /// Regenerate the lock file.
        #[clap(long)]
        relock: bool,

        /// Update all plugin sources (implies --relock).
        #[clap(long)]
        update: bool,

        /// Reinstall all plugin sources (implies --relock).
        #[clap(long, conflicts_with = "update")]
        reinstall: bool,
    },

    /// Prints detailed version information.
    Version,
}

#[derive(Debug, PartialEq, Eq, Parser)]
pub struct Add {
    /// A unique name for the binary.
    #[clap(value_name = "NAME")]
    pub name: String,

    /// The repository hosting the binary.
    #[clap(value_name = "REPO")]
    pub repo: String,

    /// The tag to use for the binary.
    #[clap(value_name = "TAG")]
    pub tag: String,

    /// A description of the binary.
    /// This is used for informational purposes only.
    #[clap(long, value_name = "DESC")]
    pub desc: Option<String>,
}

fn key_value_parser(s: &str) -> Result<(String, String), String> {
    match s.split_once('=') {
        Some((k, v)) => Ok((k.to_string(), v.to_string())),
        _ => Err(format!("{} isn't a valid key-value pair separated with =", s)),
    }
}
