use std::{
    ffi::OsString,
    io,
    os::unix::{self, ffi::OsStrExt},
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

pub fn get_file_name_utf8(path: &Path) -> anyhow::Result<Option<&str>> {
    path.file_name().map(|f| f.try_into()).transpose().map_err(Into::into)
}

pub fn detect_common_prefix(paths: impl Iterator<Item = impl AsRef<Path>>) -> Option<OsString> {
    let mut prefix = None;
    for path in paths {
        let path = path.as_ref();
        let is_dir = path.as_os_str().as_bytes().ends_with(b"/");

        trace!("detecting prefix of: {:?}", path);

        let mut components = path.components();
        let root = match (components.next(), components.next()) {
            (Some(root), None) if is_dir => root.as_os_str(),
            (Some(root), Some(_)) => root.as_os_str(),
            _ => None?,
        }
        .to_owned();

        if prefix.is_none() {
            prefix.replace(root);
        } else if prefix != Some(root) {
            None?;
        }
    }

    trace!("detected common prefix: {:?}", prefix);
    prefix
}

pub async fn load_toml<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let buf = fs::read_to_string(path).await?;
    let config = toml::from_str(&buf)?;
    Ok(config)
}
