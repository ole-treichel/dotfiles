use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};

use crate::repo::Repo;

/// Open the current branch's PR on GitHub. Falls back to the branch page when
/// there is no PR to open: at the workspace root (default branch), or when
/// `gh` finds none.
pub fn run() -> Result<()> {
    let repo = Repo::discover()?;
    let branch = repo.current_branch()?;
    let path = github_path(&repo.origin_url()?)?;
    let tree = format!("https://github.com/{path}/tree/{branch}");

    let url = if branch == repo.default_branch()? {
        tree
    } else {
        match pr_url(&path, &branch) {
            Ok(url) => url,
            Err(e) => {
                eprintln!("wt open: {e:#} — opening the branch instead");
                tree
            }
        }
    };

    open_browser(&url)?;
    println!("==> {url}");
    Ok(())
}

/// `gh pr view` by head branch. `--repo` so the answer does not depend on cwd.
fn pr_url(path: &str, branch: &str) -> Result<String> {
    let output = Command::new("gh")
        .args(["pr", "view", branch, "--repo", path, "--json", "url", "--jq", ".url"])
        .stdin(Stdio::null())
        .output()
        .context("running gh — is it installed?")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        bail!("no PR for {branch}");
    }
    Ok(url)
}

/// `git@github.com:org/repo.git` or `https://github.com/org/repo(.git)?` ->
/// `org/repo`. GitHub only — no other host needed here.
fn github_path(remote: &str) -> Result<String> {
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
    Ok(path.to_string())
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
    use super::github_path;

    #[test]
    fn converts_github_remotes() {
        assert_eq!(
            github_path("git@github.com:ole-treichel/dotfiles.git").unwrap(),
            "ole-treichel/dotfiles"
        );
        assert_eq!(
            github_path("https://github.com/ole-treichel/dotfiles.git\n").unwrap(),
            "ole-treichel/dotfiles"
        );
        assert_eq!(
            github_path("https://github.com/ole-treichel/dotfiles").unwrap(),
            "ole-treichel/dotfiles"
        );
    }

    #[test]
    fn rejects_non_github_remotes() {
        assert!(github_path("git@gitlab.com:acme/thing.git").is_err());
    }
}
