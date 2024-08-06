use std::{
    self,
    fs,
    io::{self, Read, Seek},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};

use anyhow::anyhow;
use flate2::read::GzDecoder;
use tracing::{debug, trace};
use zip::ZipArchive;

use crate::{
    context::Context,
    util::fs_ext::{detect_common_prefix, get_file_name_utf8, mkdir_p, symlink_force},
};

use super::LockedPackage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Gzip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    Plain(Option<Compression>),
    Tar(Option<Compression>),
    Zip,
}

pub fn detect_archive(path: impl AsRef<Path>) -> anyhow::Result<ArchiveKind> {
    let path = path.as_ref();
    let ext = path.extension().map(|ext| ext.try_into()).transpose()?;

    trace!("detecting archive type using extension: {:?}", ext);

    let kind = match ext {
        Some("zip") => ArchiveKind::Zip,
        Some("tar") => ArchiveKind::Tar(None),
        Some("tgz") => ArchiveKind::Tar(Some(Compression::Gzip)),
        Some("gz") => match path
            .file_stem()
            .map(Path::new)
            .and_then(|f| f.extension())
            .map(|ext| ext.try_into())
            .transpose()?
        {
            Some("tar") => ArchiveKind::Tar(Some(Compression::Gzip)),
            _ => ArchiveKind::Plain(Some(Compression::Gzip)),
        },
        _ => ArchiveKind::Plain(None),
    };

    debug!("detected archive type: {:?}", kind);

    Ok(kind)
}

pub async fn install_package(ctx: &Context, lpkg: &LockedPackage) -> anyhow::Result<()> {
    let file = &ctx.cache_dir.join(&lpkg.filename);
    let pkg = &lpkg.base;

    let install_dir = ctx.data_dir.join(format!("{}-{}", pkg.name, pkg.version));

    let archive = detect_archive(file)?;
    let file = std::fs::File::open(file)?;

    mkdir_p(&install_dir).await?;

    let link_path = ctx.bin_dir.join(&pkg.name);

    match archive {
        ArchiveKind::Plain(compression) => {
            let install_path = install_dir.join(&pkg.name);
            let install_file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .mode(0o744)
                .truncate(true)
                .open(&install_path)?;

            trace!("installing binary to: {}", install_path.display());
            let decoder: &mut dyn Read = match compression {
                Some(Compression::Gzip) => &mut GzDecoder::new(file),
                None => &mut &file,
            };

            io::copy(&mut io::BufReader::new(decoder), &mut io::BufWriter::new(install_file))?;

            symlink_force(install_path, link_path).await?;
        }
        ArchiveKind::Zip => {
            let mut archive = ZipArchive::new(file)?;
            let nfiles = archive.len();

            let prefix =
                detect_common_prefix((0..archive.len()).flat_map(|i| archive.by_index(i).ok()?.enclosed_name()));

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let path = file
                    .enclosed_name()
                    .ok_or_else(|| anyhow!("invalid filename in archive"))?;

                trace!("extracting file: {:?}", path);

                // strip the common prefix from the path
                let path = match &prefix {
                    Some(prefix) => path.strip_prefix(prefix)?,
                    None => &path,
                };

                if file.is_dir() {
                    continue;
                }

                let install_path = install_dir.join(path);
                if let Some(parent) = install_path.parent() {
                    mkdir_p(parent).await?;
                }

                trace!("installing file to: {:?}", install_path);
                let mode = file.unix_mode().unwrap_or(0o644);
                let mut output = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .mode(mode)
                    .truncate(true)
                    .open(&install_path)?;
                io::copy(&mut file, &mut output)?;

                let filename = get_file_name_utf8(path)?;
                if filename == Some(&pkg.name) || nfiles == 1 {
                    let mode = mode | 0o111;
                    fs::set_permissions(&install_path, fs::Permissions::from_mode(mode))?;
                    symlink_force(install_path, &link_path).await?;
                }
            }
        }
        ArchiveKind::Tar(compression) => {
            let build_archive = || -> anyhow::Result<tar::Archive<Box<dyn Read>>> {
                let mut src_file = file.try_clone()?;
                src_file.seek(io::SeekFrom::Start(0))?;
                Ok(tar::Archive::new(match compression {
                    Some(Compression::Gzip) => Box::new(GzDecoder::new(src_file)),
                    None => Box::new(src_file),
                }))
            };

            // detect the common prefix of all files in the archive
            let mut archive_prefix = None;
            for entry in build_archive()?.entries()? {
                let entry = entry?;
                let path = entry.path()?;

                let entry_prefix = path.components().next().map(|c| c.as_os_str().to_owned());
                archive_prefix = match archive_prefix {
                    None => entry_prefix,
                    p if p != entry_prefix => None,
                    _ => break,
                };
            }

            for entry in build_archive()?.entries()? {
                let mut entry = entry?;
                let path = entry.path()?.to_path_buf();

                trace!("extracting file: {:?}", path);

                // strip the common prefix from the path
                let path = match &archive_prefix {
                    Some(prefix) => path.strip_prefix(prefix)?,
                    None => &path,
                };

                let install_path = install_dir.join(path);
                entry.unpack(&install_path)?;

                let filename = get_file_name_utf8(path)?;
                if filename == Some(&pkg.name) {
                    let mode = entry.header().mode().unwrap_or(0o644) | 0o111;
                    fs::set_permissions(&install_path, fs::Permissions::from_mode(mode))?;
                    symlink_force(install_path, &link_path).await?;
                }
            }
        }
    };

    Ok(())
}
