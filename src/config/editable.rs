use anyhow::{bail, Context as _, Result};
use tokio::fs;

use crate::{context::Context, util::not_found_err};

use super::Package;

pub struct EditableConfig {
    ctx: Context,

    /// The parsed TOML document of the config.
    doc: toml_edit::DocumentMut,
}

impl EditableConfig {
    pub async fn load(ctx: &Context) -> Result<Self> {
        let buf = match fs::read_to_string(&ctx.config_file).await {
            Err(e) if not_found_err(&e) => include_str!("packages.toml").to_owned(),
            buf => buf.with_context(|| format!("failed to read {}", ctx.config_file.display()))?,
        };

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

        let is_default_source = pkg.source.is_default();
        let pkg = toml::to_string_pretty(pkg)
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
                for (k, v) in pkg.as_table().iter() {
                    if k == "source" && is_default_source {
                        continue;
                    }
                    table[k] = v.clone();
                }
                *item = table;
            }
            _ => bail!("package with name `{name}` already exists"),
        }

        Ok(())
    }
}
