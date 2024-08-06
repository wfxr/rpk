//! Contextual information.

mod message;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use anyhow::Error;
use serde::{Deserialize, Serialize};
pub use yansi::Color;
use yansi::Paint;

use crate::context::message::{Message, ToMessage};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Context {
    pub version: String,

    /// The location of the config file.
    pub config_file: PathBuf,

    /// The location of the configuration directory.
    pub config_dir: PathBuf,

    /// The location of the cache directory.
    pub cache_dir: PathBuf,

    /// The location of the data directory.
    pub data_dir: PathBuf,

    /// The location of the binary directory.
    pub bin_dir: PathBuf,

    #[serde(skip)]
    pub home: PathBuf,

    /// The location of the lock file.
    #[serde(skip)]
    pub lock_file: PathBuf,

    #[serde(skip)]
    pub output: Output,
}

/// The output style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Output {
    /// The requested verbosity of output.
    pub verbosity: Verbosity,
    /// Whether to not use ANSI color codes.
    pub no_color:  bool,
}

/// The requested verbosity of output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

impl Default for Verbosity {
    fn default() -> Self {
        Self::Normal
    }
}

impl Context {
    pub fn verbosity(&self) -> Verbosity {
        self.output.verbosity
    }

    /// Expands the tilde in the given path to the configured user's home
    /// directory.
    pub fn expand_tilde(&self, path: PathBuf) -> PathBuf {
        if let Ok(p) = path.strip_prefix("~") {
            self.home.join(p)
        } else {
            path
        }
    }

    /// Replaces the home directory in the given path with a tilde.
    pub fn replace_home<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Ok(p) = path.strip_prefix(&self.home) {
            Path::new("~").join(p)
        } else {
            path.to_path_buf()
        }
    }

    pub fn log_header(&self, prefix: &str, msg: impl ToMessage) {
        if self.verbosity() >= Verbosity::Normal {
            self.log_header_impl(prefix, msg.to_message(self));
        }
    }

    pub fn log_verbose_header(&self, prefix: &str, msg: impl ToMessage) {
        if self.verbosity() >= Verbosity::Verbose {
            self.log_header_impl(prefix, msg.to_message(self));
        }
    }

    fn log_header_impl(&self, prefix: &str, msg: Message<'_>) {
        if self.output.no_color {
            eprintln!("{} {}", prefix.to_uppercase(), msg);
        } else {
            eprintln!("{} {}", Paint::magenta(prefix).bold(), msg);
        }
    }

    pub fn log_status(&self, prefix: &str, msg: impl ToMessage) {
        if self.verbosity() >= Verbosity::Normal {
            self.log_impl(Color::Cyan, prefix, msg.to_message(self));
        }
    }

    pub fn log_verbose_status(&self, prefix: &str, msg: impl ToMessage) {
        if self.verbosity() >= Verbosity::Verbose {
            self.log_impl(Color::Cyan, prefix, msg.to_message(self));
        }
    }

    pub fn log_warning(&self, prefix: &str, msg: impl ToMessage) {
        if self.verbosity() >= Verbosity::Normal {
            self.log_impl(Color::Yellow, prefix, msg.to_message(self));
        }
    }

    pub fn log_verbose_warning(&self, prefix: &str, msg: impl ToMessage) {
        if self.verbosity() >= Verbosity::Verbose {
            self.log_impl(Color::Yellow, prefix, msg.to_message(self));
        }
    }

    fn log_impl(&self, color: Color, prefix: &str, msg: Message<'_>) {
        if self.output.no_color {
            eprintln!("{: >10} {}", prefix.to_uppercase(), msg);
        } else {
            eprintln!("{} {}", Paint::new(format!("{prefix: >10}")).fg(color).bold(), msg);
        }
    }

    pub fn log_error(&self, err: &Error) {
        log_error(self.output.no_color, err);
    }

    pub fn log_error_as_warning(&self, err: &Error) {
        log_error_as_warning(self.output.no_color, err);
    }
}

pub fn log_error(no_color: bool, err: &Error) {
    let pretty = prettyify_error(err);
    if no_color {
        eprintln!("\nERROR: {pretty}");
    } else {
        eprintln!("\n{} {}", Paint::red("error:").bold(), pretty);
    }
}

pub fn log_error_as_warning(no_color: bool, err: &Error) {
    let pretty = prettyify_error(err);
    if no_color {
        eprintln!("\nWARNING: {pretty}");
    } else {
        eprintln!("\n{} {}", Paint::yellow("warning:").bold(), pretty);
    }
}

fn prettyify_error(err: &Error) -> String {
    err.chain()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n  due to: ")
}
