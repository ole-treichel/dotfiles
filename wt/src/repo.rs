use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use crate::git;

pub struct Repo {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Worktree {
    pub path: PathBuf,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub bare: bool,
    pub detached: bool,
    pub prunable: bool,
    pub locked: bool,
}

impl Worktree {
    /// Directory name, which is what the user types and what `wt ls` shows.
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string())
    }

    pub fn dirty(&self) -> bool {
        if !self.path.is_dir() {
            return false;
        }
        git::out(&self.path, &["status", "--porcelain"])
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }

    /// Commits on this worktree that no remote-tracking ref contains. Asked of
    /// the branch rather than the checkout, so a worktree whose directory is
    /// already gone still answers.
    pub fn unpushed(&self, root: &Path) -> usize {
        let Some(rev) = self.branch.clone().or_else(|| self.head.clone()) else {
            return 0;
        };
        git::out(root, &["rev-list", "--count", &rev, "--not", "--remotes"])
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }
}

impl Repo {
    /// Walk up from cwd until a directory contains `.bare/`.
    pub fn discover() -> Result<Repo> {
        let cwd = env::current_dir().context("reading current directory")?;
        let mut dir: Option<&Path> = Some(cwd.as_path());
        while let Some(d) = dir {
            if d.join(".bare").is_dir() {
                return Ok(Repo { root: d.to_path_buf() });
            }
            dir = d.parent();
        }
        Err(anyhow!(
            "no .bare/ found in {} or any parent — not a wt repo (see `wt clone`)",
            cwd.display()
        ))
    }

    pub fn open(root: PathBuf) -> Repo {
        Repo { root }
    }

    pub fn fetch(&self) -> Result<()> {
        git::run(&self.root, &["fetch", "origin"])
    }

    /// `git worktree list --porcelain`, the authoritative dir↔branch map.
    pub fn worktrees(&self) -> Result<Vec<Worktree>> {
        let out = git::out(&self.root, &["worktree", "list", "--porcelain"])?;
        let mut list = Vec::new();
        let mut cur: Option<Worktree> = None;
        for line in out.lines() {
            if line.is_empty() {
                if let Some(w) = cur.take() {
                    list.push(w);
                }
                continue;
            }
            let (key, rest) = match line.split_once(' ') {
                Some((k, r)) => (k, Some(r)),
                None => (line, None),
            };
            match key {
                "worktree" => {
                    if let Some(w) = cur.take() {
                        list.push(w);
                    }
                    cur = Some(Worktree {
                        path: PathBuf::from(rest.unwrap_or_default()),
                        head: None,
                        branch: None,
                        bare: false,
                        detached: false,
                        prunable: false,
                        locked: false,
                    });
                }
                _ => {
                    let Some(w) = cur.as_mut() else { continue };
                    match key {
                        "HEAD" => w.head = rest.map(str::to_string),
                        "branch" => {
                            w.branch = rest.map(|r| r.trim_start_matches("refs/heads/").to_string())
                        }
                        "bare" => w.bare = true,
                        "detached" => w.detached = true,
                        "prunable" => w.prunable = true,
                        "locked" => w.locked = true,
                        _ => {}
                    }
                }
            }
        }
        if let Some(w) = cur.take() {
            list.push(w);
        }
        Ok(list)
    }

    /// Worktrees minus the bare entry — the ones that are actual directories.
    pub fn checkouts(&self) -> Result<Vec<Worktree>> {
        Ok(self.worktrees()?.into_iter().filter(|w| !w.bare).collect())
    }

    /// Default branch name, from `refs/remotes/origin/HEAD`.
    pub fn default_branch(&self) -> Result<String> {
        if let Ok(r) = git::out(&self.root, &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"]) {
            if let Some(b) = r.strip_prefix("origin/") {
                return Ok(b.to_string());
            }
        }
        // Not set (older clones), ask the remote once and retry.
        git::ok(&self.root, &["remote", "set-head", "origin", "-a"]);
        let r = git::out(&self.root, &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
            .context("origin/HEAD is not set; run `git remote set-head origin -a`")?;
        r.strip_prefix("origin/")
            .map(str::to_string)
            .ok_or_else(|| anyhow!("unexpected origin/HEAD: {r}"))
    }

    pub fn local_branch_exists(&self, branch: &str) -> bool {
        git::ok(
            &self.root,
            &["show-ref", "--verify", "--quiet", &format!("refs/heads/{branch}")],
        )
    }

    pub fn remote_branch_exists(&self, branch: &str) -> bool {
        git::ok(
            &self.root,
            &[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/remotes/origin/{branch}"),
            ],
        )
    }

    /// Remote branches with no worktree checked out for them.
    pub fn unchecked_remote_branches(&self) -> Result<Vec<String>> {
        let taken: Vec<String> = self
            .checkouts()?
            .into_iter()
            .filter_map(|w| w.branch)
            .collect();
        let out = git::out(
            &self.root,
            &["for-each-ref", "--format=%(refname:short)", "refs/remotes/origin"],
        )?;
        Ok(out
            .lines()
            .filter_map(|l| l.strip_prefix("origin/"))
            .filter(|b| *b != "HEAD")
            .filter(|b| !taken.iter().any(|t| t == b))
            .map(str::to_string)
            .collect())
    }

    /// A directory is free if nothing is on disk and no worktree claims it.
    pub fn ensure_dir_free(&self, slug: &str) -> Result<PathBuf> {
        let dir = self.root.join(slug);
        if dir.exists() {
            bail!("{} already exists — pick another name", dir.display());
        }
        for w in self.checkouts()? {
            if w.path == dir {
                bail!(
                    "worktree {} already exists (branch {}) — pick another name",
                    dir.display(),
                    w.branch.unwrap_or_else(|| "detached".into())
                );
            }
        }
        Ok(dir)
    }

    /// `git worktree add`, creating a tracking branch when only the remote has it.
    pub fn add_worktree(&self, dir: &Path, branch: &str) -> Result<()> {
        let dir = dir.to_string_lossy().into_owned();
        if self.local_branch_exists(branch) {
            git::run(&self.root, &["worktree", "add", &dir, branch])?;
        } else {
            let start = format!("origin/{branch}");
            git::run(
                &self.root,
                &["worktree", "add", "--track", "-b", branch, &dir, &start],
            )?;
        }
        // A branch that outlived an earlier `wt rm` lost its tracking config.
        if self.remote_branch_exists(branch)
            && !git::ok(&self.root, &["rev-parse", "--verify", &format!("{branch}@{{u}}")])
        {
            git::ok(
                &self.root,
                &[
                    "branch",
                    "--set-upstream-to",
                    &format!("origin/{branch}"),
                    branch,
                ],
            );
        }
        Ok(())
    }
}
