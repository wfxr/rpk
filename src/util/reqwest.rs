use std::{fmt, io::Write, path::Path};

use anyhow::Context;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use url::Url;

pub trait Download {
    #[allow(async_fn_in_trait)]
    async fn download(
        &self,
        url: Url,
        filename: impl AsRef<Path>,
        download_dir: impl AsRef<Path>,
        token: Option<&str>,
    ) -> anyhow::Result<()>;
}

impl Download for reqwest::Client {
    async fn download(
        &self,
        url: Url,
        filename: impl AsRef<Path>,
        download_dir: impl AsRef<Path>,
        token: Option<&str>,
    ) -> anyhow::Result<()> {
        let req = self.get(url);
        let req = match token {
            Some(token) => req.bearer_auth(token),
            None => req,
        };
        let res = req.send().await?;
        let total_size = res.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);
        pb.set_prefix("Downloading");
        pb.set_style(
            ProgressStyle::with_template("{prefix:>12.green.bold} {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({eta})")
                .context("failed to build progress style")?
                .with_key("ETA", |state: &ProgressState, w: &mut dyn fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                })
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        let mut stream = res.bytes_stream();
        let mut tmp_file = tempfile::NamedTempFile::new_in(download_dir.as_ref())?;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tmp_file.write_all(&chunk)?;
            pb.set_position(pb.position() + chunk.len() as u64);
        }
        pb.finish_and_clear();

        tmp_file.flush()?;
        tmp_file.persist(download_dir.as_ref().join(filename))?;

        Ok(())
    }
}
