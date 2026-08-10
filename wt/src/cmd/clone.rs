use std::fs;

use anyhow::{bail, Context, Result};

use crate::git;
use crate::repo::Repo;
use crate::slug::slug;

/// Build the bare-repo layout from scratch:
///
/// ```text
/// <name>/.git    -> "gitdir: ./.bare"
/// <name>/.bare   bare clone, fetching into refs/remotes/origin/*
/// <name>/<default branch>/   first worktree
/// ```
pub fn run(url: &str, name: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let name = name.unwrap_or_else(|| derive_name(url));
    if name.is_empty() {
        bail!("cannot derive a directory name from `{url}` — pass one explicitly");
    }
    let root = cwd.join(&name);
    if root.exists() {
        bail!("{} already exists", root.display());
    }

    fs::create_dir_all(&root).with_context(|| format!("creating {}", root.display()))?;
    let bare = root.join(".bare");
    git::run(&cwd, &["clone", "--bare", url, &bare.to_string_lossy()])?;
    fs::write(root.join(".git"), "gitdir: ./.bare\n")
        .with_context(|| format!("writing {}", root.join(".git").display()))?;
    git::run(
        &root,
        &[
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ],
    )?;
    git::run(&root, &["fetch", "origin"])?;
    git::ok(&root, &["remote", "set-head", "origin", "-a"]);

    let repo = Repo::open(root);
    let branch = repo.default_branch()?;
    let dir = repo.ensure_dir_free(&slug(&branch))?;
    repo.add_worktree(&dir, &branch)?;

    println!("==> {} on {branch}", dir.display());
    Ok(())
}

fn derive_name(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    let last = trimmed
        .rsplit(['/', ':'])
        .next()
        .unwrap_or_default()
        .trim_end_matches(".git");
    last.to_string()
}

#[cfg(test)]
mod tests {
    use super::derive_name;

    #[test]
    fn derives_from_url() {
        assert_eq!(derive_name("git@github.com:acme/sonax-apps.git"), "sonax-apps");
        assert_eq!(derive_name("https://github.com/acme/sonax-apps"), "sonax-apps");
        assert_eq!(derive_name("/srv/git/thing.git/"), "thing");
    }
}
