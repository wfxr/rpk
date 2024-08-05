use std::path::Path;

use futures::StreamExt;
use indicatif::ProgressStyle;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

pub trait Download {
    #[allow(async_fn_in_trait)]
    async fn download(&self, url: Url, path: impl AsRef<Path>) -> anyhow::Result<()>;
}

impl Download for reqwest::Client {
    async fn download(&self, url: Url, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let nbytes = self
            .head(url.clone())
            .send()
            .await?
            .content_length()
            .ok_or_else(|| anyhow::anyhow!("missing content length"))?;

        let pb = indicatif::ProgressBar::new(nbytes);
        let pb_style = ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-");
        pb.set_style(pb_style);

        pb.set_message(format!("Downloading {}", url));

        let mut file = File::create(path).await?;
        let mut stream = reqwest::get(url).await?.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            pb.set_position(pb.position() + chunk.len() as u64);
        }

        file.flush().await?;
        pb.finish();

        Ok(())
    }
}
