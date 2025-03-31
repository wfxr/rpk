use std::{collections::BTreeMap, fs, path::PathBuf, str};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    context::Context,
    util::{load_toml, not_found_err},
};

use super::Source;

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
    pub name: String,

    #[serde(default)]
    pub bins: Vec<String>,

    #[serde(flatten)]
    pub source: Source,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub desc: String,

    pub filename:     String,
    pub download_url: Url,
}

impl LockedConfig {
    pub fn new(ctx: Context, pkgs: BTreeMap<String, LockedPackage>) -> Self {
        Self { ctx, pkgs }
    }

    pub fn load(ctx: &Context) -> Result<Self> {
        let mut lcfg = match load_toml(&ctx.lock_file) {
            Err(e) if not_found_err(e.root_cause()) =>
                LockedConfig::new(ctx.clone(), Default::default()),
            lcfg => lcfg.with_context(|| format!("failed to load {}", ctx.lock_file.display()))?,
        };

        lcfg.ctx = ctx.clone();

        // Set the package names for convenience.
        for (name, lpkg) in lcfg.pkgs.iter_mut() {
            lpkg.name = name.clone();
            if lpkg.bins.is_empty() {
                lpkg.bins.push(name.clone());
            }
        }
        Ok(lcfg)
    }

    /// Write this `LockedConfig` to the given path.
    pub fn save(&self) -> Result<()> {
        let buf = toml::to_string_pretty(self).context("failed to serialize `LockedConfig`")?;
        fs::write(&self.ctx.lock_file, buf)
            .with_context(|| format!("failed to save {}", self.ctx.lock_file.display()))
    }

    /// Update a package in the configuration. If the package does not exist, add it.
    pub fn upsert(&mut self, lpkg: LockedPackage) {
        assert!(
            !lpkg.name.is_empty(),
            "locked package must have at least one binary"
        );
        self.pkgs.insert(lpkg.name.clone(), lpkg);
    }
}

impl LockedPackage {
    /// Get the asset path for this package.
    pub fn asset_path(&self, ctx: &Context) -> PathBuf {
        ctx.cache_dir
            .join(&self.name)
            .join(&self.version().unwrap_or("latest"))
            .join(&self.filename)
    }

    /// Get the install directory for this package.
    pub fn install_dir(&self, ctx: &Context) -> PathBuf {
        // TODO: This should be reconsidered when we support more sources.
        ctx.data_dir
            .join(&self.name)
            .join(&self.version().unwrap_or("latest"))
    }

    pub fn version(&self) -> Option<&str> {
        self.source.version()
    }
}
