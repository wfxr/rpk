use std::{
    ffi::OsString,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use anyhow::Context as _;

use super::rm_rf;

pub struct TempFile {
    temp_file: File,
    temp_path: PathBuf,
    orig_path: PathBuf,
}

impl TempFile {
    pub fn new_force(orig_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let orig_path = orig_path.as_ref().to_owned();
        let dir = orig_path.parent().context("no parent directory")?;
        let orig_name = orig_path.file_name().context("no filename")?;
        let mut temp_name = OsString::from("~");
        temp_name.push(orig_name);

        let temp_path = dir.join(&temp_name);
        rm_rf(&temp_path)?;

        let temp_file =
            File::create(&temp_path).with_context(|| format!("failed to create temporary file: {:?}", temp_path))?;
        Ok(Self { temp_file, temp_path, orig_path })
    }

    pub fn persist(self) -> io::Result<()> {
        rm_rf(&self.orig_path)?;
        fs::rename(&self.temp_path, &self.orig_path)
    }

    pub fn path(&self) -> &Path {
        &self.temp_path
    }

    pub fn file(&mut self) -> &mut File {
        &mut self.temp_file
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        rm_rf(&self.temp_path).expect("failed to delete temporary path");
    }
}
