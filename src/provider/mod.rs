mod github;

use anyhow::Result;

use crate::{
    config::{LockedPackage, Package},
    context::Context,
};

pub use github::Github;

pub trait Provider {
    fn download(&self, ctx: &Context, pkg: &Package) -> Result<LockedPackage>;
    fn download_locked(&self, ctx: &Context, pkg: &LockedPackage) -> Result<()>;
}
