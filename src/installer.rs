use std::{
    self,
    collections::{HashMap, HashSet},
    fs,
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
    util::{mkdir_p, symlink_force, Shorten},
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
    let asset_file = &lpkg.asset_path(ctx);
    let install_dir = &lpkg.install_dir(ctx);

    let archive = detect_archive(asset_file)?;
    let asset_file = fs::File::open(asset_file)?;

    if let Err(e) = fs::remove_dir_all(install_dir) {
        if e.kind() != io::ErrorKind::NotFound {
            bail!("failed to remove existing install directory: {}", e);
        }
    }
    mkdir_p(install_dir)?;

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
                Some(Compression::Gzip) => &mut GzDecoder::new(asset_file),
                None => &mut &asset_file,
            };

            io::copy(
                &mut io::BufReader::new(decoder),
                &mut io::BufWriter::new(install_file),
            )?;
        }
        ArchiveKind::Zip => {
            let mut archive = ZipArchive::new(asset_file)?;

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
                Some(Compression::Gzip) => Box::new(GzDecoder::new(asset_file)),
                None => Box::new(asset_file),
            });

            archive.unpack(install_dir)?;
        }
    };

    let mut bins_candiates: HashMap<_, _> =
        lpkg.bins.iter().map(|bin| (bin, HashSet::new())).collect();

    // Some archives contain only a single directory, move its contents to the install directory
    let files: Vec<_> = fs::read_dir(install_dir)?.try_collect()?;

    match &files[..] {
        [] => bail!("no files found in archive {}", lpkg.filename),
        [entry] if entry.path().is_file() => {
            bins_candiates.values_mut().for_each(|candidates| {
                candidates.insert(entry.path().to_owned());
            });
        }
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

    for entry in WalkDir::new(install_dir)
        .into_iter()
        .filter_ok(|entry| entry.path().is_file())
    {
        let entry = entry?;

        let file_name = entry.file_name().to_string_lossy().to_string();
        if let Some(candidates) = bins_candiates.get_mut(&file_name) {
            trace!(
                "found bin candidate: {}",
                entry.path().strip_prefix(install_dir)?.display()
            );
            candidates.insert(entry.path().to_owned());
        }
    }

    for (bin, candidates) in &bins_candiates {
        let link = ctx.bin_dir.join(bin);

        let (path, meta) = candidates
            .iter()
            .filter_map(|c| Some((c, fs::metadata(c).ok()?)))
            .max_by_key(|(_, meta)| meta.len())
            .ok_or_else(|| anyhow!("no binary {} found in archive", bin))?;

        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(path, perms)?;

        symlink_force(path, &link)?;
        debug!("link built: '{}' -> '{}'", link.display(), path.display());

        if candidates.len() > 1 {
            ctx.log_warning(
                "Warning",
                format!(
                    "{} candidate binaries found in archive, using the largest one: '{}'",
                    candidates.len(),
                    path.shorten()?
                ),
            );
        }
    }

    Ok(())
}
