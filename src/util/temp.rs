use std::{
    ffi,
    fs,
    io,
    path::{Path, PathBuf},
    result,
};

use anyhow::Result;

/// Holds a temporary directory or file path that is removed when dropped.
pub struct TempPath {
    /// The temporary directory or file path.
    path: PathBuf,
}

impl TempPath {
    /// Create a new `TempPath` based on an original path, the temporary
    /// filename will be placed in the same directory with a deterministic name.
    ///
    /// # Errors
    ///
    /// If the temporary path already exists.
    pub fn new(original_path: &Path) -> result::Result<Self, PathBuf> {
        let mut path = original_path.parent().unwrap().to_path_buf();
        let mut file_name = ffi::OsString::from("~");
        file_name.push(original_path.file_name().unwrap());
        path.push(file_name);
        if path.exists() {
            Err(path)
        } else {
            Ok(Self { path })
        }
    }

    /// Create a new `TempPath` based on an original path, if something exists
    /// at that temporary path is will be deleted.
    pub fn new_force(original_path: &Path) -> Result<Self> {
        match Self::new(original_path) {
            Ok(temp) => Ok(temp),
            Err(path) => {
                rm_rf(&path)?;
                Ok(Self { path })
            }
        }
    }

    /// Access the underlying `Path`.
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Move the temporary path to a new location.
    pub fn rename(self, new_path: &Path) -> io::Result<()> {
        rm_rf(new_path)?;
        fs::rename(&self.path, new_path)
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        rm_rf(&self.path).expect("failed to delete temporary path");
    }
}

/// Remove a file or directory.
fn rm_rf(path: &Path) -> io::Result<()> {
    let res = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };

    res.or_else(|e| match e.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(e),
    })
}
