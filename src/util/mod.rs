mod build;
mod fs;

pub mod reqwest;

pub use build::*;
pub use fs::*;

use std::{error::Error, io};

pub fn not_found_err(e: &(dyn Error + 'static)) -> bool {
    matches!(e.downcast_ref::<io::Error>(), Some(e) if e.kind() == io::ErrorKind::NotFound)
}
