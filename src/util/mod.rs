pub mod build;
pub mod fs_ext;
mod temp;

use std::{error::Error, io};

pub use crate::util::temp::TempPath;

pub fn not_found_err(e: &(dyn Error + 'static)) -> bool {
    matches!(e.downcast_ref::<io::Error>(), Some(e) if e.kind() == io::ErrorKind::NotFound)
}
