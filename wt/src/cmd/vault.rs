use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use crate::config;
use crate::picker;
use crate::repo::{Repo, Worktree};

/// Symlink `docs/<slug>/` from selected worktrees into the Obsidian vault.
pub fn run(slugs: &[String]) -> Result<()> {
    let repo = Repo::discover()?;
    let checkouts = repo.checkouts()?;
    let vault = config::load()?.vault;

    let targets = if slugs.is_empty() {
        pick(&repo, &checkouts)?
    } else {
        slugs
            .iter()
            .map(|s| resolve(&checkouts, s))
            .collect::<Result<Vec<_>>>()?
    };
    if targets.is_empty() {
        return Ok(());
    }

    let repo_name = repo
        .root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| repo.root.display().to_string());

    // Each slug is independent: one bad symlink target must not stop the rest.
    let mut failed = false;
    for w in &targets {
        match link_one(&repo_name, &vault, w) {
            Ok(dest) => println!("==> {}", dest.display()),
            Err(e) => {
                eprintln!("wt vault: {}: {e:#}", w.name());
                failed = true;
            }
        }
    }
    if failed {
        bail!("some worktrees failed to link");
    }
    Ok(())
}

fn link_one(repo_name: &str, vault: &Path, w: &Worktree) -> Result<PathBuf> {
    let source = w.path.join("docs").join(w.name());
    if !source.is_dir() {
        bail!("no {} to link", source.display());
    }
    let dest = vault.join(format!("{repo_name}-{}", w.name()));
    // ln -sfn semantics: a symlink (even dangling) is relinked; anything else
    // already at that name is a real vault file/folder and must not be eaten.
    if dest.is_symlink() {
        std::fs::remove_file(&dest)
            .with_context(|| format!("removing existing symlink {}", dest.display()))?;
    } else if dest.exists() {
        bail!("{} already exists and is not a symlink", dest.display());
    }
    symlink(&source, &dest)
        .with_context(|| format!("linking {} -> {}", dest.display(), source.display()))?;
    Ok(dest)
}

fn pick(repo: &Repo, checkouts: &[Worktree]) -> Result<Vec<Worktree>> {
    let eligible: Vec<&Worktree> = checkouts
        .iter()
        .filter(|w| w.path.join("docs").join(w.name()).is_dir())
        .collect();
    if eligible.is_empty() {
        println!("no worktrees in {} have a docs/<slug>/ to link", repo.root.display());
        return Ok(Vec::new());
    }
    let items: Vec<picker::Item> = eligible
        .iter()
        .map(|w| {
            picker::Item::new(w.name())
                .secondary(w.branch.clone().unwrap_or_else(|| "(detached)".into()))
        })
        .collect();
    let picked = picker::pick("link to vault", &items, true)?;
    Ok(picked.into_iter().map(|i| eligible[i].clone()).collect())
}

fn resolve(checkouts: &[Worktree], arg: &str) -> Result<Worktree> {
    let wanted = PathBuf::from(arg);
    let canonical = wanted.canonicalize().ok();
    let hit = checkouts.iter().find(|w| {
        w.name() == arg
            || w.path == wanted
            || canonical.as_ref().is_some_and(|c| &w.path == c)
    });
    hit.cloned().ok_or_else(|| anyhow!("no worktree `{arg}` — `wt ls` lists them"))
}
