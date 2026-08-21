use std::env;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};

use crate::repo::Repo;

/// Open the repo on GitHub: the checked-out branch when run from inside a
/// worktree, the default branch when run from the workspace root.
pub fn run() -> Result<()> {
    let repo = Repo::discover()?;
    let branch = current_branch(&repo)?;
    let web_root = github_web_root(&repo.origin_url()?)?;
    let url = format!("{web_root}/tree/{branch}");

    open_browser(&url)?;
    println!("==> {url}");
    Ok(())
}

fn current_branch(repo: &Repo) -> Result<String> {
    let cwd = env::current_dir().context("reading current directory")?;
    let cwd = cwd.canonicalize().unwrap_or(cwd);
    let root = repo.root.canonicalize().unwrap_or_else(|_| repo.root.clone());
    if cwd == root {
        return repo.default_branch();
    }
    for w in repo.checkouts()? {
        let path = w.path.canonicalize().unwrap_or_else(|_| w.path.clone());
        if cwd == path || cwd.starts_with(&path) {
            let name = w.name();
            return w
                .branch
                .ok_or_else(|| anyhow!("{name} is on a detached HEAD, not a branch"));
        }
    }
    bail!("{} is not a known worktree — `wt ls` lists them", cwd.display());
}

/// `git@github.com:org/repo.git` or `https://github.com/org/repo(.git)?` ->
/// `https://github.com/org/repo`. GitHub only — no other host needed here.
fn github_web_root(remote: &str) -> Result<String> {
    let remote = remote.trim();
    let path = remote
        .strip_prefix("git@github.com:")
        .or_else(|| remote.strip_prefix("ssh://git@github.com/"))
        .or_else(|| remote.strip_prefix("https://github.com/"))
        .or_else(|| remote.strip_prefix("http://github.com/"))
        .ok_or_else(|| anyhow!("origin `{remote}` is not a github.com remote"))?;
    let path = path.trim_end_matches(".git").trim_end_matches('/');
    if path.is_empty() {
        bail!("origin `{remote}` has no org/repo path");
    }
    Ok(format!("https://github.com/{path}"))
}

fn open_browser(url: &str) -> Result<()> {
    Command::new("xdg-open")
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("running xdg-open — is it installed?")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::github_web_root;

    #[test]
    fn converts_github_remotes() {
        assert_eq!(
            github_web_root("git@github.com:ole-treichel/dotfiles.git").unwrap(),
            "https://github.com/ole-treichel/dotfiles"
        );
        assert_eq!(
            github_web_root("https://github.com/ole-treichel/dotfiles.git\n").unwrap(),
            "https://github.com/ole-treichel/dotfiles"
        );
        assert_eq!(
            github_web_root("https://github.com/ole-treichel/dotfiles").unwrap(),
            "https://github.com/ole-treichel/dotfiles"
        );
    }

    #[test]
    fn rejects_non_github_remotes() {
        assert!(github_web_root("git@gitlab.com:acme/thing.git").is_err());
    }
}
