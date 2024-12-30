mod github;

use anyhow::Result;

use crate::config::{LockedPackage, Package};

pub use github::Github;

pub trait Provider {
    fn lock(&self, pkg: &Package) -> Result<LockedPackage>;
    fn download(&self, pkg: &LockedPackage) -> Result<()>;
}
