use std::{
    process,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::{Context as _, Result};
use inquire::{Select, Text};
use tracing::debug;

use crate::{
    commands,
    config::{Config, EditableConfig, LockedConfig, Package, Source},
    context::Context,
    manager::{restore_package, restore_packages, sync_package, sync_packages, SyncResult},
    provider::Github,
};

pub async fn add(ctx: &Context, mut pkg: Package) -> Result<()> {
    let mut ecfg = EditableConfig::load(ctx).await?;
    ctx.log_verbose_header_p("Loaded", &ctx.config_file);

    let (lpkg, _) = sync_package(ctx, &pkg, None, false).await?;
    pkg.desc = lpkg.desc.clone();

    ecfg.upsert(&pkg)?;

    let mut lcfg = LockedConfig::load(ctx).await?;
    lcfg.upsert(lpkg);

    ecfg.save().await?;
    lcfg.save().await?;
    ctx.log_verbose_header_p("Locked", &ctx.lock_file);

    Ok(())
}

pub async fn sync(ctx: &Context) -> Result<(), anyhow::Error> {
    let cfg = Config::load(ctx).await?;
    ctx.log_verbose_header_p("Loaded", &ctx.config_file);
    let mut lcfg = LockedConfig::load(ctx).await?;
    ctx.log_verbose_header_p("Loaded", &ctx.lock_file);

    sync_packages(ctx, &cfg, &mut lcfg).await?;

    lcfg.save().await?;
    ctx.log_verbose_header_p("Locked", &ctx.lock_file);

    Ok(())
}

pub async fn restore(ctx: &Context, package: Option<String>) -> Result<(), anyhow::Error> {
    let lcfg = LockedConfig::load(ctx).await?;
    ctx.log_verbose_header_p("Loaded", &ctx.lock_file);

    match package {
        Some(pkg) => {
            let lpkg = lcfg
                .pkgs
                .get(&pkg)
                .with_context(|| format!("package {} not found", pkg))?;
            restore_package(ctx, lpkg).await?;
        }
        None => restore_packages(lcfg).await?,
    }

    Ok(())
}

pub async fn update(ctx: &Context, package: Option<String>) -> Result<(), anyhow::Error> {
    let cfg = Config::load(ctx).await?;
    ctx.log_verbose_header_p("Loaded", &ctx.config_file);
    match package {
        Some(package) => {
            let pkg = cfg
                .pkgs
                .values()
                .find(|pkg| pkg.name == package)
                .cloned()
                .with_context(|| format!("package {} not found", package))?;

            let mut lcfg = LockedConfig::load(ctx).await?;
            let old_lpkg = lcfg.pkgs.get(&package);

            // Sync the package.
            let (new_lpkg, sync_res) = sync_package(ctx, &pkg, old_lpkg, true).await?;

            // Update the package in the lock file.
            if sync_res == SyncResult::Updated {
                lcfg.upsert(new_lpkg);
                lcfg.save().await?;
                ctx.log_verbose_header_p("Locked", &ctx.lock_file);
            }
        }
        None => {
            let mut lcfg = LockedConfig::load(ctx).await?;
            ctx.log_verbose_header_p("Loaded", &ctx.lock_file);

            let mut updated = false;
            for old_lpkg in lcfg.pkgs.clone().values() {
                let pkg = match cfg.pkgs.get(&old_lpkg.name) {
                    Some(pkg) => pkg,
                    None => continue,
                };

                // Sync the package.
                let (new_lpkg, sync_res) = sync_package(ctx, pkg, Some(old_lpkg), true).await?;

                // Update the package in the lock file.
                if sync_res == SyncResult::Updated {
                    lcfg.upsert(new_lpkg);
                    updated = true;
                }
            }

            if updated {
                lcfg.save().await?;
                ctx.log_verbose_header_p("Locked", &ctx.lock_file);
            }
        }
    };
    Ok(())
}

pub async fn search(query: String, top: u8, ctx: Context) -> Result<(), anyhow::Error> {
    let gh = Github::new()?;
    let repos = gh.search_repo(&query, top).await?;

    let stars_width = Arc::new(AtomicUsize::new(0));
    let fullname_width = Arc::new(AtomicUsize::new(0));

    // Items list
    let items: Vec<_> = repos
        .into_iter()
        .flat_map(|repo| {
            Some(RepoItem {
                name:           repo.name,
                desc:           repo.description.unwrap_or_default(),
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
            false => Some(answer.desc),
            true => None,
        },
    };

    debug!("selected: {:?}", pkg);
    commands::add(&ctx, pkg).await?;

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
