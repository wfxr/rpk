use anyhow::{Context as _, Result};
use itertools::Itertools as _;
use skim::{prelude::*, SkimItemReceiver, SkimItemSender};
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
    ctx.log_header("Loaded", ctx.config_file.as_path());

    let (lpkg, _) = sync_package(ctx, &pkg, None).await?;
    pkg.desc = lpkg.desc.clone();

    ecfg.upsert(&pkg)?;

    let mut lcfg = LockedConfig::load(ctx).await?;
    lcfg.upsert(lpkg);

    ecfg.save().await?;
    lcfg.save().await?;
    ctx.log_header("Locked", ctx.lock_file.as_path());

    Ok(())
}

pub async fn sync(ctx: &Context) -> Result<(), anyhow::Error> {
    let cfg = Config::load(ctx).await?;
    ctx.log_header("Loaded", ctx.config_file.as_path());

    let lcfg = sync_packages(ctx, cfg).await?;

    lcfg.save().await?;
    ctx.log_header("Locked", ctx.lock_file.as_path());

    Ok(())
}

pub async fn restore(ctx: &Context, package: Option<String>) -> Result<(), anyhow::Error> {
    let lcfg = LockedConfig::load(ctx).await?;
    ctx.log_header("Loaded", ctx.lock_file.as_path());

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
    ctx.log_header("Loaded", ctx.config_file.as_path());
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
            let (new_lpkg, sync_res) = sync_package(ctx, &pkg, old_lpkg).await?;

            // Update the package in the lock file.
            if sync_res == SyncResult::Updated {
                lcfg.upsert(new_lpkg);
                lcfg.save().await?;
                ctx.log_header("Locked", ctx.lock_file.as_path());
            }
        }
        None => {
            let mut lcfg = LockedConfig::load(ctx).await?;
            ctx.log_header("Loaded", ctx.lock_file.as_path());

            let mut updated = false;
            for pkg in cfg.pkgs.values() {
                let old_lpkg = lcfg.pkgs.get(&pkg.name);

                // Sync the package.
                let (new_lpkg, sync_res) = sync_package(ctx, pkg, old_lpkg).await?;

                // Update the package in the lock file.
                if sync_res == SyncResult::Updated {
                    lcfg.upsert(new_lpkg);
                    updated = true;
                }
            }

            if updated {
                lcfg.save().await?;
                ctx.log_header("Locked", ctx.lock_file.as_path());
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

    // Header line
    let header = RepoSkimItem {
        name:           "Name".to_string(),
        desc:           "Description".to_string(),
        stars:          "Stars".to_string(),
        fullname:       "Repository".to_string(),
        stars_width:    stars_width.clone(),
        fullname_width: fullname_width.clone(),
    };

    // Items list
    let list = repos
        .into_iter()
        .flat_map(|repo| {
            Some(RepoSkimItem {
                name:           repo.name,
                desc:           repo.description.unwrap_or_default(),
                stars:          repo.stargazers_count.map(|x| x.to_string()).unwrap_or_default(),
                stars_width:    stars_width.clone(),
                fullname:       repo.full_name?,
                fullname_width: fullname_width.clone(),
            })
        })
        .rev()
        .chain(std::iter::once(header))
        .inspect(|item| {
            stars_width.fetch_max(item.stars.len(), Ordering::Relaxed);
            fullname_width.fetch_max(item.fullname.len(), Ordering::Relaxed);
        })
        .collect_vec();

    let height = (list.len() + 1).min(25).to_string();

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    for item in list.into_iter().rev() {
        tx.send(Arc::new(item))?;
    }
    drop(tx);

    let skip_opts = SkimOptionsBuilder::default()
        .inline_info(true)
        .header_lines(1)
        .prompt(Some("Select a package > "))
        .height(Some(&height))
        .multi(false)
        .reverse(true)
        .exit0(true)
        .no_clear(true)
        .no_clear_start(true)
        .build()?;

    let output = match Skim::run_with(&skip_opts, Some(rx)) {
        Some(output) => match output.is_abort {
            false => output,
            true => std::process::exit(130),
        },
        None => std::process::exit(135),
    };

    let selected = output.selected_items.into_iter().map(|repo| {
        let repo = (*repo)
            .as_any()
            .downcast_ref::<RepoSkimItem>()
            .expect("something wrong with downcast");
        Package {
            name:    repo.name.clone(),
            source:  Source::Github { repo: repo.fullname.clone() },
            version: None,
            desc:    if repo.desc.is_empty() {
                None
            } else {
                Some(repo.desc.clone())
            },
        }
    });

    for pkg in selected {
        debug!("selected: {:?}", pkg);
        commands::add(&ctx, pkg).await?;
    }

    Ok(())
}

struct RepoSkimItem {
    name:           String,
    fullname:       String,
    desc:           String,
    stars:          String,
    stars_width:    Arc<AtomicUsize>,
    fullname_width: Arc<AtomicUsize>,
}

impl SkimItem for RepoSkimItem {
    fn text(&self) -> Cow<str> {
        let Self { stars, fullname, desc, .. } = self;
        let stars_width = self.stars_width.load(Ordering::Relaxed);
        let fullname_width = self.fullname_width.load(Ordering::Relaxed);
        Cow::Owned(format!("{stars:>stars_width$}  {fullname:fullname_width$}  {desc}",))
    }
}
