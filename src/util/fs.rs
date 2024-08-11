use std::{
    io,
    os::unix::{self},
    path::Path,
};

use anyhow::Result;
use serde::Deserialize;
use tokio::fs;
use tracing::trace;

pub async fn symlink_force(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if let Err(e) = fs::remove_file(dst).await {
        if e.kind() != io::ErrorKind::NotFound {
            return Err(e.into());
        }
    }

    unix::fs::symlink(src, dst).map_err(Into::into)
}

pub async fn mkdir_p(dir: impl AsRef<Path>) -> anyhow::Result<()> {
    trace!("creating directory: {:?}", dir.as_ref());
    match fs::create_dir_all(dir).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn load_toml<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let buf = fs::read_to_string(path).await?;
    let config = toml::from_str(&buf)?;
    Ok(config)
}
