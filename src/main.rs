pub mod cli;
pub mod commands;
pub mod config;
pub mod context;
pub mod installer;
pub mod manager;
pub mod provider;
pub mod util;

use std::process;

use anyhow::Context as _;
use clap::{CommandFactory as _, Parser as _};
use clap_complete::{generate, generate_to};
use cli::{Opt, SubCommand, ENV_BIN_DIR, ENV_CACHE_DIR, ENV_CONFIG_DIR, ENV_DATA_DIR};
use config::{Package, Source};
use context::{log_error, Context};
use tracing_subscriber::EnvFilter;
use util::{mkdir_p, Shorten as _, CRATE_NAME};

fn try_main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let output = opt.output_opt();

    let Opt { bin_dir, data_dir, cache_dir, config_dir, command, .. } = opt;

    let xdg_dirs = xdg::BaseDirectories::with_prefix(CRATE_NAME)?;

    let config_dir = config_dir.unwrap_or_else(|| xdg_dirs.get_config_home());
    mkdir_p(&config_dir).context("failed to create config dir")?;

    let cache_dir = cache_dir.unwrap_or_else(|| xdg_dirs.get_cache_home());
    mkdir_p(&cache_dir).context("failed to create cache dir")?;

    let data_dir = data_dir.unwrap_or_else(|| xdg_dirs.get_data_home().join("packages"));
    mkdir_p(&data_dir).context("failed to create data dir")?;

    let bin_dir = bin_dir.unwrap_or_else(|| xdg_dirs.get_data_home().join("bin"));
    mkdir_p(&bin_dir).context("failed to create binary dir")?;

    let config_file = config_dir.join("packages.toml");
    let lock_file = config_dir.join("packages.lock");

    let version = util::CRATE_RELEASE.to_string();
    let ctx = Context {
        version,
        config_file,
        config_dir,
        cache_dir,
        data_dir,
        bin_dir,
        lock_file,
        output,
    };

    macro_rules! with_flock {
        ($command:expr) => {
            let _guard = acquire_flock(&ctx)?;
            $command
        };
    }

    match command {
        SubCommand::Init { from } => {
            with_flock!(commands::init(&ctx, from)?);
        }
        SubCommand::List => {
            with_flock!(commands::list(&ctx)?);
        }
        SubCommand::Sync => {
            with_flock!(commands::sync(&ctx)?);
        }
        SubCommand::Update { package } => {
            with_flock!(commands::update(&ctx, package)?);
        }
        SubCommand::Restore { package } => {
            with_flock!(commands::restore(&ctx, package)?);
        }
        SubCommand::Find { query, top } => {
            with_flock!(commands::find(query, top, &ctx)?);
        }
        SubCommand::Add { name, repo: (owner, repo), version, desc } => {
            let pkg = Package {
                name: name.unwrap_or_else(|| repo.clone()),
                source: Source::Github { repo: format!("{}/{}", owner, repo) },
                version,
                desc,
            };
            with_flock!(commands::add(&ctx, pkg)?);
        }

        SubCommand::Env => {
            macro_rules! print_env {
                ($name:expr, $path:expr) => {
                    println!(r#"export {}="{}""#, $name, $path.shorten()?);
                };
            }
            print_env!(ENV_CONFIG_DIR, ctx.config_dir);
            print_env!(ENV_CACHE_DIR, ctx.cache_dir);
            print_env!(ENV_DATA_DIR, ctx.data_dir);
            print_env!(ENV_BIN_DIR, ctx.bin_dir);
            println!(r#"export PATH="${ENV_BIN_DIR}:$PATH""#);
        }
        SubCommand::Completions { shell, dir } => {
            let cmd = &mut Opt::command();
            match dir {
                Some(dir) => {
                    let path = generate_to(shell, cmd, cmd.get_name().to_string(), dir)?;
                    ctx.log_status("Generated", path.shorten()?);
                }
                None => generate(shell, cmd, cmd.get_name().to_string(), &mut std::io::stdout()),
            }
        }
        SubCommand::Version => {
            println!("{} {}", util::CRATE_NAME, util::CRATE_VERBOSE_VERSION);
        }
    }

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().with_file(true).with_line_number(true))
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if let Err(e) = try_main() {
        log_error(true, &e);
        process::exit(1);
    }
}

fn acquire_flock(ctx: &Context) -> anyhow::Result<fmutex::Guard> {
    let path = &ctx.config_dir;
    match fmutex::try_lock(path).with_context(|| format!("failed to open `{}`", path.display()))? {
        Some(g) => Ok(g),
        None => {
            ctx.log_warning("Blocking", format!("waiting for file lock on {}", path.shorten()?));
            fmutex::lock(path).with_context(|| format!("failed to acquire file lock `{}`", path.display()))
        }
    }
}
