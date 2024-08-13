use std::{
    fs,
    process,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::{bail, Context as _, Result};
use inquire::{Select, Text};
use itertools::Itertools;
use rayon::prelude::*;
use tabled::{
    settings::{object::Rows, Color, Padding, Style},
    Table,
    Tabled,
};
use tracing::debug;
use url::Url;

use crate::{
    commands,
    config::{Config, EditableConfig, LockedConfig, Package, Source},
    context::Context,
    manager::{restore_package, restore_packages, sync_package, sync_packages},
    provider::Github,
    util::{remove_file_if_exists, Emojify},
};

pub fn init(ctx: &Context, from: Option<Url>) -> Result<()> {
    if ctx.config_file.exists() {
        bail!("config file already exists: {}", ctx.config_file.display());
    }

    remove_file_if_exists(&ctx.lock_file).with_context(|| {
        format!(
            "failed to remove lock file {}",
            ctx.replace_home(&ctx.lock_file).display(),
        )
    })?;

    match from {
        Some(url) => {
            let body = ureq::get(url.as_str()).call()?.into_string()?;
            debug!("fetched config file: {}", body);
            // Parse and validate the downloaded config file.
            toml::from_str::<Config>(&body)?;
            fs::write(&ctx.config_file, body)?;
        }
        None => {
            Config::load(ctx)?;
        }
    }

    ctx.log_header_p("Initialized", &ctx.config_file);
    sync(ctx)
}

pub fn list(ctx: &Context) -> Result<(), anyhow::Error> {
    let lcfg = LockedConfig::load(ctx)?;
    ctx.log_verbose_header_p("Loaded", &ctx.lock_file);

    #[derive(Debug, Tabled)]
    #[tabled(rename_all = "UPPERCASE")]
    struct Item {
        pkg:         String,
        version:     String,
        description: String,
    }

    let items = lcfg
        .pkgs
        .into_values()
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .map(|lpkg| Item {
            pkg:         lpkg.name,
            version:     lpkg.version,
            description: lpkg.desc.map(|s| s.emojify()).unwrap_or_default(),
        });

    let mut table = Table::new(items);
    table
        .with(Style::empty())
        .modify(Rows::first(), Color::BOLD)
        .with(Padding::new(0, 4, 0, 0));
    println!("{table}");

    Ok(())
}

pub fn add(ctx: &Context, mut pkg: Package) -> Result<()> {
    let mut ecfg = EditableConfig::load(ctx)?;
    ctx.log_verbose_header_p("Loaded", &ctx.config_file);

    let lpkg = sync_package(ctx, &pkg, None, false)?;
    pkg.desc = lpkg.desc.clone();

    ecfg.upsert(&pkg)?;

    let mut lcfg = LockedConfig::load(ctx)?;
    lcfg.upsert(lpkg);

    ecfg.save()?;
    lcfg.save()?;
    ctx.log_verbose_header_p("Locked", &ctx.lock_file);

    Ok(())
}

pub fn sync(ctx: &Context) -> Result<(), anyhow::Error> {
    let cfg = Config::load(ctx)?;
    ctx.log_verbose_header_p("Loaded", &ctx.config_file);
    let mut lcfg = LockedConfig::load(ctx)?;
    ctx.log_verbose_header_p("Loaded", &ctx.lock_file);

    sync_packages(ctx, &cfg, &mut lcfg)?;

    lcfg.save()?;
    ctx.log_verbose_header_p("Locked", &ctx.lock_file);

    Ok(())
}

pub fn restore(ctx: &Context, package: Option<String>) -> Result<(), anyhow::Error> {
    let lcfg = LockedConfig::load(ctx)?;
    ctx.log_verbose_header_p("Loaded", &ctx.lock_file);

    match package {
        Some(pkg) => {
            let lpkg = lcfg
                .pkgs
                .get(&pkg)
                .with_context(|| format!("package {} not found", pkg))?;
            restore_package(ctx, lpkg)?;
        }
        None => restore_packages(lcfg)?,
    }

    Ok(())
}

pub fn update(ctx: &Context, package: Option<String>) -> Result<(), anyhow::Error> {
    let cfg = Config::load(ctx)?;
    ctx.log_verbose_header_p("Loaded", &ctx.config_file);
    match package {
        Some(package) => {
            let pkg = cfg
                .pkgs
                .values()
                .find(|pkg| pkg.name == package)
                .cloned()
                .with_context(|| format!("package {} not found", package))?;

            let mut lcfg = LockedConfig::load(ctx)?;
            let old_lpkg = lcfg.pkgs.get(&package);

            // Sync the package.
            let new_lpkg = sync_package(ctx, &pkg, old_lpkg, true)?;

            // Update the package in the lock file.
            lcfg.upsert(new_lpkg);
            lcfg.save()?;
            ctx.log_verbose_header_p("Locked", &ctx.lock_file);
        }
        None => {
            let mut lcfg = LockedConfig::load(ctx)?;
            ctx.log_verbose_header_p("Loaded", &ctx.lock_file);

            lcfg.pkgs
                .clone()
                .into_par_iter()
                .filter_map(|(_, lpkg)| cfg.pkgs.get(&lpkg.name).map(|pkg| (pkg, lpkg)))
                .map(|(pkg, old_lpkg)| sync_package(ctx, pkg, Some(&old_lpkg), true))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .for_each(|res| {
                    lcfg.upsert(res);
                });

            lcfg.save()?;
            ctx.log_verbose_header_p("Locked", &ctx.lock_file);
        }
    };
    Ok(())
}

pub fn find(query: String, top: u8, ctx: &Context) -> Result<(), anyhow::Error> {
    let gh = Github::new(ctx.clone())?;
    let repos = gh.search_repo(&query, top)?;

    let stars_width = Arc::new(AtomicUsize::new(0));
    let fullname_width = Arc::new(AtomicUsize::new(0));

    // Items list
    let items: Vec<_> = repos
        .into_iter()
        .flat_map(|repo| {
            Some(RepoItem {
                name:           repo.name,
                desc:           repo.description.unwrap_or_default().emojify(),
                stars:          repo.stargazers_count.map(|x| format!("★ {x}")).unwrap_or_default(),
                stars_width:    stars_width.clone(),
                fullname:       repo.full_name?,
                fullname_width: fullname_width.clone(),
            })
        })
        .inspect(|item| {
            stars_width.fetch_max(item.stars.len(), Ordering::Relaxed);
            fullname_width.fetch_max(item.fullname.len(), Ordering::Relaxed);
        })
        .collect();

    let page_size = items.len().min(25);
    let answer = Select::new("Select a package ", items)
        .with_page_size(page_size)
        .prompt();

    use inquire::InquireError::*;
    let answer = match answer {
        Err(OperationCanceled | OperationInterrupted) => process::exit(1),
        answer => answer?,
    };

    let name = Text::new("Choose package name?")
        .with_initial_value(&answer.name)
        .prompt()?;

    let pkg = Package {
        name,
        source: Source::Github { repo: answer.fullname },
        version: None,
        desc: match answer.desc.is_empty() {
            false => Some(answer.desc.emojify()),
            true => None,
        },
    };

    debug!("selected: {:?}", pkg);
    commands::add(ctx, pkg)?;

    Ok(())
}

struct RepoItem {
    name:           String,
    fullname:       String,
    desc:           String,
    stars:          String,
    stars_width:    Arc<AtomicUsize>,
    fullname_width: Arc<AtomicUsize>,
}

impl std::fmt::Display for RepoItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { stars, fullname, desc, .. } = self;
        let stars_width = self.stars_width.load(Ordering::Relaxed);
        let fullname_width = self.fullname_width.load(Ordering::Relaxed);
        f.write_fmt(format_args!("{stars:stars_width$}  {fullname:fullname_width$}  {desc}",))
    }
}
