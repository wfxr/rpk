use std::{io::Write, path::Path};

use anyhow::anyhow;
use futures::StreamExt;
use indicatif::ProgressStyle;
use url::Url;

pub trait Download {
    #[allow(async_fn_in_trait)]
    async fn download(
        &self,
        url: Url,
        download_dir: impl AsRef<Path>,
        filename: impl AsRef<Path>,
    ) -> anyhow::Result<()>;
}

impl Download for reqwest::Client {
    async fn download(
        &self,
        url: Url,
        download_dir: impl AsRef<Path>,
        filename: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let mut tmp_file = tempfile::NamedTempFile::new_in(download_dir.as_ref())?;
        let nbytes = self
            .head(url.clone())
            .send()
            .await?
            .content_length()
            .ok_or_else(|| anyhow!("missing content length"))?;

        let pb = indicatif::ProgressBar::new(nbytes);
        let pb_style = ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-");
        pb.set_style(pb_style);

        pb.set_message(format!("Downloading {}", url));

        let mut stream = reqwest::get(url).await?.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tmp_file.write_all(&chunk)?;
            pb.set_position(pb.position() + chunk.len() as u64);
        }

        pb.finish();

        tmp_file.flush()?;
        tmp_file.persist(download_dir.as_ref().join(filename))?;

        Ok(())
    }
}
