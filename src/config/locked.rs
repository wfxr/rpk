use std::{collections::BTreeMap, str};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
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
    pub name:         String,
    pub version:      String,
    #[serde(flatten)]
    pub source:       Source,
    pub desc:         Option<String>,
    pub filename:     String,
    pub download_url: Option<Url>,
}

impl LockedConfig {
    pub fn new(ctx: Context, pkgs: BTreeMap<String, LockedPackage>) -> Self {
        Self { ctx, pkgs }
    }

    pub async fn load(ctx: &Context) -> Result<Self> {
        let mut lcfg = match load_toml(&ctx.lock_file).await {
            Err(e) if not_found_err(e.root_cause()) => LockedConfig::new(ctx.clone(), Default::default()),
            lcfg => lcfg.context(format!("failed to load {}", ctx.lock_file.display()))?,
        };

        lcfg.ctx = ctx.clone();

        // Set the package names for convenience.
        for (name, lpkg) in lcfg.pkgs.iter_mut() {
            lpkg.name = name.clone();
        }
        Ok(lcfg)
    }

    /// Write this `LockedConfig` to the given path.
    pub async fn save(&self) -> Result<()> {
        let buf = toml::to_string_pretty(self).context("failed to serialize `LockedConfig`")?;
        fs::write(&self.ctx.lock_file, buf)
            .await
            .with_context(|| format!("failed to save {}", self.ctx.lock_file.display()))
    }

    /// Update a package in the configuration. If the package does not exist, add it.
    pub fn upsert(&mut self, lpkg: LockedPackage) {
        self.pkgs.insert(lpkg.name.clone(), lpkg);
    }
}
