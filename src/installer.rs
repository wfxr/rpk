use std::{
    self,
    ffi::OsString,
    fs::{self},
    io::{self, Read},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use anyhow::{anyhow, bail};
use flate2::read::GzDecoder;
use itertools::Itertools;
use tar::Archive as TarArchive;
use tracing::{debug, trace};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{
    config::LockedPackage,
    context::Context,
    util::{mkdir_p, symlink_force},
};

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

pub fn install_package(ctx: &Context, lpkg: &LockedPackage) -> anyhow::Result<()> {
    let file = &ctx.cache_dir.join(&lpkg.filename);
    let install_dir = ctx.data_dir.join(&lpkg.name).join(&lpkg.version);

    let archive = detect_archive(file)?;
    let file = std::fs::File::open(file)?;

    if let Err(e) = fs::remove_dir_all(&install_dir) {
        if e.kind() != io::ErrorKind::NotFound {
            bail!("failed to remove existing install directory: {}", e);
        }
    }
    mkdir_p(&install_dir)?;

    let link_path = ctx.bin_dir.join(&lpkg.name);

    match archive {
        ArchiveKind::Plain(compression) => {
            let install_path = install_dir.join(&lpkg.name);
            let install_file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&install_path)?;

            trace!("installing binary to: {}", install_path.display());
            let decoder: &mut dyn Read = match compression {
                Some(Compression::Gzip) => &mut GzDecoder::new(file),
                None => &mut &file,
            };

            io::copy(&mut io::BufReader::new(decoder), &mut io::BufWriter::new(install_file))?;
        }
        ArchiveKind::Zip => {
            let mut archive = ZipArchive::new(file)?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let path = file
                    .enclosed_name()
                    .ok_or_else(|| anyhow!("invalid filename in archive"))?;

                trace!("extracting file: {:?}", path);

                if file.is_dir() {
                    continue;
                }

                let install_path = install_dir.join(&path);
                if let Some(parent) = install_path.parent() {
                    mkdir_p(parent)?;
                }

                trace!("installing file to: {:?}", install_path);
                let mut output = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&install_path)?;
                io::copy(&mut file, &mut output)?;
            }
        }
        ArchiveKind::Tar(compression) => {
            let mut archive: TarArchive<Box<dyn Read>> = TarArchive::new(match compression {
                Some(Compression::Gzip) => Box::new(GzDecoder::new(file)),
                None => Box::new(file),
            });

            archive.unpack(&install_dir)?;
        }
    };

    let mut bin_candidates = Vec::new();

    // Some archives contain only a single directory, move its contents to the install directory
    let files: Vec<_> = fs::read_dir(&install_dir)?.try_collect()?;

    match &files[..] {
        [] => bail!("no files found in archive {}", lpkg.filename),
        [entry] if entry.path().is_file() => bin_candidates.push(entry.path()),
        [entry] if entry.path().is_dir() =>
            for entry in fs::read_dir(entry.path())? {
                let path = entry?.path();
                let name = path
                    .file_name()
                    .ok_or_else(|| anyhow!("invalid filename: {:?}", path.file_name()))?;
                let install_path = install_dir.join(name);

                trace!("trim extra prefix: {:?} -> {:?}", path, install_path);
                fs::rename(&path, &install_path)?;
            },
        _ => (),
    }

    if bin_candidates.is_empty() {
        for entry in WalkDir::new(&install_dir)
            .into_iter()
            .filter_ok(|entry| entry.path().is_file())
        {
            let entry = entry?;

            let pkg_name: OsString = lpkg.name.clone().into();
            if entry.file_name() == pkg_name {
                trace!(
                    "found bin candidate: {}",
                    entry.path().strip_prefix(&install_dir)?.display()
                );
                bin_candidates.push(entry.path().to_owned());
            }
        }
    }

    match &bin_candidates[..] {
        [] => bail!("no binary found in archive"),
        [path, ..] => {
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            fs::set_permissions(path, perms)?;

            symlink_force(path, &link_path)?;
            debug!("link built: '{}' -> '{}'", path.display(), link_path.display());

            if bin_candidates.len() > 1 {
                ctx.log_warning(
                    "Warning",
                    format!(
                        "Multiple binaries found in archive, using the first one: '{}'",
                        ctx.replace_home(path).display()
                    ),
                );
            }
        }
    }

    Ok(())
}
