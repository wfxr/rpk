mod editable;
mod locked;
mod raw;

pub use editable::EditableConfig;
pub use locked::{LockedConfig, LockedPackage};
pub use raw::{GitHubSource, GithubSourceType, RawConfig, RawPackage};
use url::Url;

use std::{collections::BTreeMap, fmt, str};

use anyhow::Result;
use serde::{
    Deserialize,
    Serialize,
};

use crate::context::Context;

#[derive(Debug, Clone)]
pub struct Config {
    pub pkgs: BTreeMap<String, Package>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name:    String,
    pub enabled: bool,
    pub bins:    Vec<String>,
    pub source:  Source,
    pub desc:    String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    pub owner: String,
    pub name:  String,
}

impl fmt::Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

/// The source for a [`Package`].
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Source {
    /// A clonable Git repository.
    Git { url: Url, reference: Option<GitReference> },
    /// A remote file.
    Remote { url: Url },
    /// A release from a GitHub repository.
    Release { repo: Repository, tag: Option<String> },
}

impl Source {
    pub fn version(&self) -> Option<&str> {
        match self {
            Source::Git { reference, .. } => reference.as_ref().map(|r| r.as_ref()),
            Source::Release { tag, .. } => tag.as_deref(),
            _ => None,
        }
    }
}

/// A Git reference.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GitReference {
    /// From the tip of a branch.
    Branch(String),
    /// From a specific revision.
    Rev(String),
    /// From a tag.
    #[serde(alias = "version")]
    Tag(String),
}

impl AsRef<str> for GitReference {
    fn as_ref(&self) -> &str {
        match self {
            GitReference::Branch(b) | GitReference::Rev(b) | GitReference::Tag(b) => b,
        }
    }
}

impl Config {
    /// Load the configuration from the given path.
    pub fn load(ctx: &Context) -> Result<Self> {
        let raw_cfg = RawConfig::load(ctx)?;

        let pkgs = raw_cfg
            .pkgs
            .into_iter()
            .map(|(name, p)| Ok((name, p.try_into()?)))
            .collect::<Result<_>>()?;

        Ok(Self { pkgs })
    }
}
