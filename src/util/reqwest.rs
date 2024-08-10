use std::{fmt, io::Write, path::Path};

use anyhow::{anyhow, Context};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::{header, IntoUrl, Method, RequestBuilder};
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

    fn with_token(&self, method: Method, url: impl IntoUrl, token: Option<&str>) -> RequestBuilder;
}

impl Download for reqwest::Client {
    fn with_token(&self, method: Method, url: impl IntoUrl, token: Option<&str>) -> RequestBuilder {
        match token {
            Some(token) => self.request(method, url).bearer_auth(token),
            None => self.request(method, url),
        }
    }

    async fn download(
        &self,
        url: Url,
        filename: impl AsRef<Path>,
        download_dir: impl AsRef<Path>,
        token: Option<&str>,
    ) -> anyhow::Result<()> {
        let total_size = self
            .with_token(Method::HEAD, url.clone(), token)
            .send()
            .await?
            .headers()
            .get(header::CONTENT_LENGTH)
            .ok_or_else(|| anyhow!("missing content length"))?
            .to_str()?
            .parse::<u64>()?;

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template("{prefix:>12.green.bold} {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({eta})")
                .context("failed to build progress style")?
                .with_key("ETA", |state: &ProgressState, w: &mut dyn fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                })
                .progress_chars("#>-"),
        );
        pb.set_prefix("Downloading");

        let mut stream = self.with_token(Method::GET, url, token).send().await?.bytes_stream();

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
