mod github;

use anyhow::Result;

use crate::{
    config::{LockedPackage, Package},
    context::Context,
};

pub use github::Github;

#[allow(async_fn_in_trait)]
pub trait Provider {
    async fn download(&self, ctx: &Context, pkg: &Package) -> Result<LockedPackage>;
    async fn download_locked(&self, ctx: &Context, pkg: &LockedPackage) -> Result<()>;
}
