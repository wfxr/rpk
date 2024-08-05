use std::path::PathBuf;

use anyhow::Result;

use crate::{config::Package, context::Context};

pub mod github;
pub mod reqwest_ext;

pub use github::Github;

pub trait Provider {
    #[allow(async_fn_in_trait)]
    async fn download(&self, ctx: Context, pkg: &Package) -> Result<PathBuf>;
}
