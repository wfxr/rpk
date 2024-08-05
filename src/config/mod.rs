//! The user configuration.

use std::{fmt, str};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    pub pkgs: Vec<Package>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "source")]
#[serde(rename_all = "lowercase")]
pub struct Package {
    pub name:    String,
    pub version: String,
    #[serde(flatten)]
    pub source:  Source,

    pub desc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "source")]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Github { repo: String },
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Github { repo } => {
                write!(f, "github.com:{}", repo)
            }
        }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { name, version, source, desc: _ } = self;
        write!(f, "{name}@{version} from {source}")
    }
}
