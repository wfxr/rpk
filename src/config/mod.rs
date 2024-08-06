//! The user configuration.

use std::{fmt, str};

use anyhow::{Context as _, Result};
use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize,
    Deserializer,
    Serialize,
};
use tokio::fs;

use crate::{context::Context, util::fs_ext::load_toml};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(default)]
    pub pkgs: Vec<Package>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Package {
    pub name:    String,
    pub version: String,
    #[serde(flatten)]
    pub source:  Source,

    pub desc: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(tag = "source")]
#[serde(rename_all = "snake_case")]
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

impl<'de> Deserialize<'de> for Source {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SourceVisitor)
    }
}

struct SourceVisitor;

impl<'de> Visitor<'de> for SourceVisitor {
    type Value = Source;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("enum Source")
    }
    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Source,
            Repo,
        }
        const FIELDS: &[&str] = &["github"];

        let (mut source, mut repo) = (None, None);
        while let Some(key) = map.next_key()? {
            match key {
                Field::Source => match source {
                    None => source = Some(map.next_value()?),
                    Some(_) => return Err(Error::duplicate_field("source")),
                },
                Field::Repo => match repo {
                    None => repo = Some(map.next_value()?),
                    Some(_) => return Err(Error::duplicate_field("repo")),
                },
            }
        }

        let source = match source.unwrap_or("github".to_owned()).as_str() {
            "github" => Source::Github { repo: repo.ok_or_else(|| Error::missing_field("repo"))? },
            s => return Err(Error::unknown_variant(s, FIELDS)),
        };

        Ok(source)
    }
}

impl Config {
    /// Load the configuration from the given path.
    pub async fn load(ctx: &Context) -> Result<Self> {
        load_toml(&ctx.config_file)
            .await
            .with_context(|| format!("failed to load {}", ctx.config_file.display()))
    }

    /// Write this `LockedConfig` to the given path.
    pub async fn save(&self, ctx: &Context) -> Result<()> {
        let buf = toml::to_string_pretty(self).context("failed to serialize `Config`")?;
        fs::write(&ctx.config_file, buf)
            .await
            .with_context(|| format!("failed to save {}", ctx.config_file.display()))
    }

    /// Add a package to the configuration.
    pub fn add_pkg(&mut self, pkg: Package) {
        self.pkgs.push(pkg);
    }
}
