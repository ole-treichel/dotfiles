use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};

use crate::picker;
use crate::repo::{Repo, Worktree};
use crate::slug::slug;
use crate::{git, tms};

/// Remove worktrees and their local branches. Remote branches are never touched.
pub fn run(dirs: &[String], force: bool, yes: bool) -> Result<()> {
    let repo = Repo::discover()?;
    let checkouts = repo.checkouts()?;
    if checkouts.is_empty() {
        println!("no worktrees to remove");
        return Ok(());
    }

    let targets = if dirs.is_empty() {
        pick(&repo, &checkouts)?
    } else {
        dirs.iter()
            .map(|d| resolve(&checkouts, d))
            .collect::<Result<Vec<_>>>()?
    };
    if targets.is_empty() {
        return Ok(());
    }

    if !force {
        let mut blocked = Vec::new();
        for w in &targets {
            let mut why = Vec::new();
            if w.dirty() {
                why.push("uncommitted changes".to_string());
            }
            let ahead = w.unpushed(&repo.root);
            if ahead > 0 {
                why.push(format!("{ahead} unpushed commit{}", plural(ahead)));
            }
            if !why.is_empty() {
                blocked.push(format!("  {}: {}", w.name(), why.join(", ")));
            }
        }
        if !blocked.is_empty() {
            bail!(
                "refusing to remove:\n{}\nuse --force to remove anyway",
                blocked.join("\n")
            );
        }
    }

    if !yes && !confirm(&targets)? {
        println!("nothing removed");
        return Ok(());
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    for w in &targets {
        let path = w.path.to_string_lossy().into_owned();
        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        args.push(&path);
        git::run(&repo.root, &args)?;
        println!("==> removed {}", w.name());

        if let Some(branch) = &w.branch {
            // -D, not -d: git only knows HEAD and the upstream, so it refuses
            // branches that are on origin but have no tracking config. The
            // check above already proved every commit lives on a remote.
            match git::try_run(&repo.root, &["branch", "-D", branch])? {
                Ok(()) => println!("==> deleted branch {branch}"),
                Err(e) => eprintln!("warning: kept branch {branch}: {e}"),
            }
        }
        if cwd.starts_with(&w.path) {
            eprintln!("warning: your shell is still inside the removed {}", w.name());
        }
    }

    git::run(&repo.root, &["worktree", "prune"])?;
    tms::refresh();
    Ok(())
}

/// Show what is about to go and wait for a y/N. Deleting a worktree is the one
/// thing here that cannot be undone from the CLI.
fn confirm(targets: &[Worktree]) -> Result<bool> {
    println!("about to remove:");
    let width = targets.iter().map(|w| w.name().len()).max().unwrap_or(0);
    for w in targets {
        println!(
            "  {:<width$}  {}",
            w.name(),
            w.branch.clone().unwrap_or_else(|| "(detached)".into())
        );
    }
    println!("and their local branches — remote branches are kept");

    print!(
        "remove {} worktree{}? [y/N] ",
        targets.len(),
        plural(targets.len())
    );
    stdout().flush()?;
    let mut answer = String::new();
    if stdin().read_line(&mut answer)? == 0 {
        println!();
        bail!("no answer on stdin — pass --yes to remove without confirming");
    }
    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn pick(repo: &Repo, checkouts: &[Worktree]) -> Result<Vec<Worktree>> {
    // The default branch's directory holds the .env files the hooks seed from,
    // so it is never offered up for removal — `wt rm main` still works.
    let keep = slug(&repo.default_branch().unwrap_or_default());
    let choices: Vec<&Worktree> = checkouts.iter().filter(|w| w.name() != keep).collect();
    if choices.is_empty() {
        println!("nothing to remove besides {keep}/");
        return Ok(Vec::new());
    }
    let items: Vec<picker::Item> = choices
        .iter()
        .map(|w| {
            let mut item = picker::Item::new(w.name())
                .secondary(w.branch.clone().unwrap_or_else(|| "(detached)".into()));
            if w.dirty() {
                item = item.tag("dirty", picker::Tone::Warn);
            }
            let ahead = w.unpushed(&repo.root);
            if ahead > 0 {
                item = item.tag(format!("↑{ahead}"), picker::Tone::Warn);
            }
            if w.prunable {
                item = item.tag("prunable", picker::Tone::Muted);
            }
            item
        })
        .collect();
    let picked = picker::pick("remove worktrees", &items, true)?;
    Ok(picked.into_iter().map(|i| choices[i].clone()).collect())
}

fn resolve(checkouts: &[Worktree], arg: &str) -> Result<Worktree> {
    let wanted = PathBuf::from(arg);
    let canonical = wanted.canonicalize().ok();
    let hit = checkouts.iter().find(|w| {
        w.name() == arg
            || w.path == wanted
            || canonical.as_ref().is_some_and(|c| &w.path == c)
    });
    match hit {
        Some(w) => Ok(w.clone()),
        None => bail!("no worktree `{arg}` — `wt ls` lists them"),
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
