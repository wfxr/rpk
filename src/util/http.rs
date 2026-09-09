use std::{
    io::{BufWriter, Read, Write},
    path::Path,
};

use ureq::{
    Body,
    SendBody,
    http::{Request, Response, header::AUTHORIZATION},
    middleware::{Middleware, MiddlewareNext},
};
use url::Url;

use super::temp::TempFile;

pub struct BearerAuthMiddleware(pub Option<String>);

impl Middleware for BearerAuthMiddleware {
    fn handle(
        &self,
        mut request: Request<SendBody>,
        next: MiddlewareNext,
    ) -> Result<Response<Body>, ureq::Error> {
        if let Some(token) = &self.0 {
            let mut value = format!("Bearer {token}")
                .parse::<ureq::http::HeaderValue>()
                .map_err(ureq::http::Error::from)?;
            value.set_sensitive(true);
            request.headers_mut().insert(AUTHORIZATION, value);
        }

        next.handle(request)
    }
}

pub trait UreqExt {
    fn download(&self, url: &Url, path: impl AsRef<Path>) -> anyhow::Result<()>;
}

impl UreqExt for ureq::Agent {
    fn download(&self, url: &Url, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut reader = self.get(url.as_str()).call()?.into_body().into_reader();
        let mut tmp_file = TempFile::new_force(path.as_ref())?;
        {
            let mut writer = BufWriter::new(tmp_file.file());
            let mut buf = [0; 4 * 1024];
            loop {
                let nread = reader.read(&mut buf)?;
                // pb.inc(nread as u64);
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
}

pub fn http_get(url: Url) -> anyhow::Result<String> {
    let mut resp = ureq::get(url.as_str()).call()?;
    Ok(resp.body_mut().read_to_string()?)
}
