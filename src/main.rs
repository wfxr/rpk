#![feature(anonymous_lifetime_in_impl_trait)]
pub mod config;
pub mod context;
pub mod lock;
pub mod provider;
pub mod util;

use std::{
    env::consts::{ARCH, OS},
    fs,
    process,
};

use anyhow::anyhow;
use config::Config;
use context::{log_error, Context, Output};
use lock::lock_packages;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use util::build::{self, CRATE_NAME};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().with_file(true).with_line_number(true))
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    debug!("OS: {OS}, Arch: {ARCH}");

    let output = Output::default();
    let home = match home::home_dir() {
        Some(home) => home,
        None => {
            let err = anyhow!("failed to determine the current user's home directory");
            log_error(output.no_color, &err);
            process::exit(1);
        }
    };

    let xdg_dirs = xdg::BaseDirectories::with_prefix(CRATE_NAME)?;

    let ctx = Context {
        version: build::CRATE_RELEASE.to_string(),
        home,
        config_dir: xdg_dirs.get_config_home(),
        config_file: xdg_dirs.place_config_file("config.toml")?,
        lock_file: xdg_dirs.place_config_file("config.lock")?,
        cache_dir: xdg_dirs.get_cache_home(),
        data_dir: xdg_dirs.get_data_home().join("packages"),
        bin_dir: xdg_dirs.get_data_home().join("bin"),
        output,
    };
    debug!("context: {:#?}", ctx);

    ctx.init()?;

    let config = fs::read_to_string(&ctx.config_file)?;
    let config: Config = toml::from_str(&config)?;
    ctx.log_header("Loaded", ctx.config_file.as_path());

    let locked_config = lock_packages(&ctx, config).await?;
    locked_config.save()?;

    ctx.log_header("Locked", ctx.config_file.as_path());

    Ok(())
}
