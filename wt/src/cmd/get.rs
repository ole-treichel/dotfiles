use anyhow::{bail, Result};

use crate::hooks::{self, HookCtx};
use crate::picker;
use crate::repo::Repo;
use crate::slug::slug;
use crate::tms;

/// Check out existing remote branches. Branches keep their name verbatim, only
/// the directory is slugged.
pub fn run(branch: Option<String>) -> Result<()> {
    let repo = Repo::discover()?;
    repo.fetch()?;

    let branches = match branch {
        Some(b) => vec![b],
        None => {
            let candidates = repo.unchecked_remote_branches()?;
            if candidates.is_empty() {
                println!("every remote branch already has a worktree");
                return Ok(());
            }
            let picked = picker::pick("get branches", &candidates, true)?;
            picked.into_iter().map(|i| candidates[i].clone()).collect()
        }
    };
    if branches.is_empty() {
        return Ok(());
    }

    let default_branch = repo.default_branch()?;
    let mut result = Ok(());
    for branch in branches {
        result = checkout(&repo, branch, &default_branch);
        if result.is_err() {
            break;
        }
    }
    tms::refresh();
    result
}

fn checkout(repo: &Repo, branch: String, default_branch: &str) -> Result<()> {
    if !repo.remote_branch_exists(&branch) && !repo.local_branch_exists(&branch) {
        bail!("no branch `{branch}` on origin — `wt new {branch}` creates one");
    }
    for w in repo.checkouts()? {
        if w.branch.as_deref() == Some(branch.as_str()) {
            bail!("branch `{branch}` is already checked out in {}", w.path.display());
        }
    }

    let name = slug(&branch);
    if name.is_empty() {
        bail!("branch `{branch}` slugs to nothing");
    }
    let dir = repo.ensure_dir_free(&name)?;

    repo.add_worktree(&dir, &branch)?;
    println!("==> {} on {branch}", dir.display());

    hooks::run_post_create(&HookCtx {
        event: "get",
        repo_root: repo.root.clone(),
        dir,
        base_ref: format!("origin/{branch}"),
        branch,
        slug: name,
        default_branch: default_branch.to_string(),
    })
}
