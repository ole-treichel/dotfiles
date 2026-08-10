use anyhow::{bail, Result};

use crate::hooks::{self, HookCtx};
use crate::repo::Repo;
use crate::slug::slug;
use crate::{git, tms};

/// Create a branch + worktree from the base ref, then run the post-create hooks.
pub fn run(words: &[String], from: Option<String>) -> Result<()> {
    let repo = Repo::discover()?;
    let name = slug(&words.join(" "));
    if name.is_empty() {
        bail!("`{}` slugs to nothing — give it some letters", words.join(" "));
    }

    repo.fetch()?;
    let default_branch = repo.default_branch()?;
    let base = from.unwrap_or_else(|| format!("origin/{default_branch}"));
    git::out(&repo.root, &["rev-parse", "--verify", &format!("{base}^{{commit}}")])
        .map_err(|_| anyhow::anyhow!("base ref `{base}` does not resolve to a commit"))?;

    if repo.local_branch_exists(&name) {
        bail!("branch `{name}` already exists — use `wt get {name}` to check it out");
    }
    let dir = repo.ensure_dir_free(&name)?;

    // --no-track: branching off origin/main would otherwise leave main as the
    // upstream until the hook pushes. The push in 30-commit-push-pr.sh sets it.
    git::run(
        &repo.root,
        &[
            "worktree",
            "add",
            "--no-track",
            "-b",
            &name,
            &dir.to_string_lossy(),
            &base,
        ],
    )?;
    println!("==> {} on {name} (from {base})", dir.display());

    let result = hooks::run_post_create(&HookCtx {
        event: "new",
        repo_root: repo.root.clone(),
        dir,
        branch: name.clone(),
        slug: name,
        base_ref: base,
        default_branch,
    });
    tms::refresh();
    result
}
