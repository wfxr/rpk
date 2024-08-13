use std::{
    fs,
    io,
    os::unix::{self},
    path::Path,
};

use anyhow::Result;
use serde::Deserialize;
use tracing::trace;

pub fn symlink_force(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if let Err(e) = fs::remove_file(dst) {
        if e.kind() != io::ErrorKind::NotFound {
            return Err(e.into());
        }
    }

    unix::fs::symlink(src, dst).map_err(Into::into)
}

pub fn mkdir_p(dir: impl AsRef<Path>) -> anyhow::Result<()> {
    trace!("creating directory: {:?}", dir.as_ref());
    match fs::create_dir_all(dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn load_toml<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let buf = fs::read_to_string(path)?;
    let config = toml::from_str(&buf)?;
    Ok(config)
}

pub fn remove_file_if_exists(path: impl AsRef<Path>) -> Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}
