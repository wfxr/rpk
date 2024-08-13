use std::{
    fmt,
    io::{BufWriter, Write},
    path::Path,
};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use url::Url;

use super::temp::TempFile;

pub fn download(url: Url, path: impl AsRef<Path>, token: Option<&str>) -> anyhow::Result<()> {
    let req = ureq::get(url.as_str()).set("User-Agent", "ureq");
    let req = match token {
        Some(token) => req.set("Authorization", &format!("Bearer {}", token)),
        None => req,
    };

    let resp = req.call()?;

    let total_bytes: u64 = resp.header("Content-Length").unwrap_or("0").parse()?;

    let pb = ProgressBar::new(total_bytes);
    pb.set_prefix("Downloading");
    pb.set_style(
        ProgressStyle::with_template("{prefix:>12.green.bold} {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({eta})")
            .expect("failed to build progress style")
            .with_key("ETA", |state: &ProgressState, w: &mut dyn fmt::Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            })
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    let mut reader = resp.into_reader();

    let mut tmp_file = TempFile::new_force(path.as_ref())?;
    {
        let mut writer = BufWriter::new(tmp_file.file());
        let mut buf = [0; 4 * 1024];
        loop {
            let nread = reader.read(&mut buf)?;
            pb.inc(nread as u64);
            if nread == 0 {
                break;
            }
            writer.write_all(&buf[..nread])?;
        }
        writer.flush()?;
    }
    tmp_file.persist()?;

    Ok(())
}

pub fn http_get(url: Url) -> anyhow::Result<String> {
    let resp = ureq::get(url.as_str()).call()?;
    Ok(resp.into_string()?)
}
