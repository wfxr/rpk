use std::{fmt, fs::File, io::Write, path::Path};

use curl::easy::{self, WriteError};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use tracing::error;
use url::Url;

use super::temp::TempPath;

/// Download a remote file.
pub fn download(url: Url, path: impl AsRef<Path>, token: Option<&str>) -> anyhow::Result<()> {
    let mut easy = curl::easy::Easy::new();
    easy.useragent(&format!("curl/{}", curl::Version::get().version()))?;
    easy.fail_on_error(true)?;
    easy.follow_location(true)?;
    if let Some(token) = token {
        let mut list = easy::List::new();
        list.append(&format!("Authorization: Bearer {}", token))?;
        easy.http_headers(list)?;
    }
    easy.url(url.as_ref())?;
    easy.progress(true)?;
    let mut transfer = easy.transfer();

    let tmp_path = TempPath::new_force(path.as_ref())?;

    let mut pb = None;
    transfer.progress_function(move |total, done, _, _| {
        if total > 0.0 {
            let pb = pb.get_or_insert_with(|| {
                let pb = ProgressBar::new(total as u64);
                pb.set_prefix("Downloading");
                pb.set_style(
                    ProgressStyle::with_template(
                        "{prefix:>12.green.bold} {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({eta})",
                    )
                    .expect("failed to build progress style")
                    .with_key("ETA", |state: &ProgressState, w: &mut dyn fmt::Write| {
                        write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                    })
                    .progress_chars("█▉▊▋▌▍▎▏  "),
                );
                pb
            });
            pb.set_length(total as u64);
            pb.set_position(done as u64);
        }
        true
    })?;

    let mut tmp_file = File::create(tmp_path.path())?;
    transfer.write_function(move |data| match tmp_file.write_all(data) {
        Ok(()) => Ok(data.len()),
        Err(e) => {
            error!("failed to write to file: {}", e);
            Err(WriteError::Pause)
        }
    })?;
    transfer.perform()?;

    tmp_path.rename(path.as_ref())?;
    Ok(())
}

pub fn http_get(url: Url) -> anyhow::Result<String> {
    let mut easy = curl::easy::Easy::new();
    easy.useragent(&format!("curl/{}", curl::Version::get().version()))?;
    easy.fail_on_error(true)?;
    easy.follow_location(true)?;
    easy.url(url.as_ref())?;
    let mut buf = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            buf.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    Ok(String::from_utf8(buf)?)
}
