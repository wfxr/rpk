//! The user configuration.

use std::{collections::BTreeMap, fmt, str};

use anyhow::{bail, Context as _, Result};
use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize,
    Deserializer,
    Serialize,
};
use tokio::fs;
use url::Url;

use crate::{context::Context, util::fs_ext::load_toml};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(default)]
    pub pkgs: BTreeMap<String, Package>,
}

pub struct EditableConfig {
    ctx: Context,

    /// The parsed TOML version of the config.
    doc: toml_edit::DocumentMut,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Package {
    #[serde(skip)]
    pub name:    String,
    pub version: Option<String>,
    #[serde(flatten)]
    pub source:  Source,
    pub desc:    Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(tag = "source")]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Github { repo: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct LockedConfig {
    #[serde(flatten)]
    pub ctx: Context,

    #[serde(default)]
    pub pkgs: BTreeMap<String, LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "source")]
#[serde(rename_all = "lowercase")]
pub struct LockedPackage {
    #[serde(skip)]
    pub name:         String,
    pub version:      String,
    #[serde(flatten)]
    pub source:       Source,
    pub desc:         Option<String>,
    pub filename:     String,
    pub download_url: Option<Url>,
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
        write!(
            f,
            "{name}@{version} from {source}",
            version = version.as_deref().unwrap_or("latest"),
        )
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
        let mut cfg: Self = load_toml(&ctx.config_file)
            .await
            .with_context(|| format!("failed to load {}", ctx.config_file.display()))?;

        // Set the package names for convenience.
        for (name, pkg) in cfg.pkgs.iter_mut() {
            pkg.name = name.clone();
        }

        Ok(cfg)
    }
}

impl From<LockedPackage> for Package {
    fn from(val: LockedPackage) -> Self {
        Package {
            name:    val.name,
            version: val.version.into(),
            source:  val.source,
            desc:    val.desc,
        }
    }
}

impl EditableConfig {
    pub async fn load(ctx: &Context) -> Result<Self> {
        let buf = fs::read_to_string(&ctx.config_file)
            .await
            .with_context(|| format!("failed to read {}", ctx.config_file.display()))?;
        let ctx = ctx.clone();
        let doc = buf.parse().context("failed to parse TOML")?;
        Ok(Self { ctx, doc })
    }

    pub async fn save(&self) -> Result<()> {
        fs::write(&self.ctx.config_file, self.doc.to_string())
            .await
            .with_context(|| format!("failed to write {}", self.ctx.config_file.display()))
    }

    pub fn upsert(&mut self, pkg: &Package) -> Result<()> {
        let name = &pkg.name;

        let mini = toml::to_string_pretty(pkg)
            .context("failed to serialize package")?
            .parse::<toml_edit::DocumentMut>()
            .context("failed to serialized package")?;

        match &mut self.doc["pkgs"] {
            item @ toml_edit::Item::None => {
                let mut pkgs = toml_edit::Table::new();
                pkgs.set_implicit(true);
                *item = toml_edit::Item::Table(pkgs);
            }
            toml_edit::Item::Table(_) => {}
            _ => bail!("current `pkgs` entry is not a table"),
        }

        match &mut self.doc["pkgs"][&name] {
            item @ toml_edit::Item::None => {
                let mut table = toml_edit::table();
                for (k, v) in mini.as_table().iter() {
                    table[k] = v.clone();
                }
                *item = table;
            }
            _ => bail!("plugin with name `{name}` already exists"),
        }

        Ok(())
    }
}

impl LockedConfig {
    pub fn new(ctx: Context, pkgs: BTreeMap<String, LockedPackage>) -> Self {
        Self { ctx, pkgs }
    }
}
