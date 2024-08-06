use std::path::PathBuf;

use anyhow::Result;

use crate::{config::Package, context::Context, lock::LockedPackage};

pub mod github;
pub mod reqwest_ext;

pub use github::Github;

#[allow(async_fn_in_trait)]
pub trait Provider {
    async fn download(&self, ctx: Context, pkg: &Package) -> Result<PathBuf>;
    async fn download_locked(&self, ctx: Context, pkg: &LockedPackage) -> Result<PathBuf>;
}
