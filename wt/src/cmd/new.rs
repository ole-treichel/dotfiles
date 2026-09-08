use std::io::{stdin, stdout, Write};

use anyhow::{bail, Result};

use crate::hooks::{self, HookCtx};
use crate::picker;
use crate::repo::Repo;
use crate::slug::slug;
use crate::{git, moco, tms};

/// `feat`, `chore`, `fix` and nothing else, with what each one is for.
const PREFIXES: [(&str, &str); 3] = [
    ("feat", "new feature"),
    ("chore", "maintenance"),
    ("fix", "bugfix"),
];

/// Create a branch + worktree from the base ref, then run the post-create hooks.
///
/// No words: the wizard walks prefix → description → MOCO project and builds
/// `<prefix>-<description>-p<number>`. Words: they are slugged and used as
/// given — the escape hatch for scratch branches, which get no prefix and no
/// project number.
pub fn run(words: &[String], from: Option<String>) -> Result<()> {
    // Before the wizard: walking three steps only to be told there is no
    // `.bare/` here would waste every one of them.
    let repo = Repo::discover()?;

    let name = if words.is_empty() {
        match wizard()? {
            Some(name) => name,
            None => {
                println!("nothing created");
                return Ok(());
            }
        }
    } else {
        let name = slug(&words.join(" "));
        if name.is_empty() {
            bail!("`{}` slugs to nothing — give it some letters", words.join(" "));
        }
        name
    };

    create(&repo, &name, from)
}

fn create(repo: &Repo, name: &str, from: Option<String>) -> Result<()> {
    repo.fetch()?;
    let default_branch = repo.default_branch()?;
    let base = from.unwrap_or_else(|| format!("origin/{default_branch}"));
    git::out(&repo.root, &["rev-parse", "--verify", &format!("{base}^{{commit}}")])
        .map_err(|_| anyhow::anyhow!("base ref `{base}` does not resolve to a commit"))?;

    if repo.local_branch_exists(name) {
        bail!("branch `{name}` already exists — use `wt get {name}` to check it out");
    }
    let dir = repo.ensure_dir_free(name)?;

    // --no-track: branching off origin/main would otherwise leave main as the
    // upstream until the hook pushes. The push in 30-commit-push-pr.sh sets it.
    git::run(
        &repo.root,
        &[
            "worktree",
            "add",
            "--no-track",
            "-b",
            name,
            &dir.to_string_lossy(),
            &base,
        ],
    )?;
    println!("==> {} on {name} (from {base})", dir.display());

    let result = hooks::run_post_create(&HookCtx {
        event: "new",
        repo_root: repo.root.clone(),
        dir,
        branch: name.to_string(),
        slug: name.to_string(),
        base_ref: base,
        default_branch,
    });
    tms::refresh();
    result
}

/// The three steps in the order the name reads, so the preview only ever grows
/// to the right. `None` means cancelled.
fn wizard() -> Result<Option<String>> {
    // One takeover for all three steps. Opening and closing the alternate
    // screen per step flashes the shell in between.
    let mut screen = picker::Session::open()?;

    let items: Vec<picker::Item> = PREFIXES
        .iter()
        .map(|(prefix, what)| picker::Item::new(*prefix).secondary(*what))
        .collect();
    let Some(&chosen) = screen.pick("prefix", &items, false)?.first() else {
        return Ok(None);
    };
    let prefix = PREFIXES[chosen].0;

    // Loops rather than erroring: a stray enter should redraw the step, not
    // end the run. `esc` is the way out, same as in every picker.
    let description = loop {
        let title = format!("{prefix}-<description>-p…");
        let Some(typed) = screen.prompt(&title, "type the description")? else {
            return Ok(None);
        };
        let slugged = slug(&typed);
        if !slugged.is_empty() {
            break slugged;
        }
    };

    let projects = moco::projects()?;
    let rows: Vec<picker::Item> = projects
        .iter()
        .map(|p| {
            // Customer first: it is the coarser sort, so the left edge of the
            // column groups the list by client as you scan it.
            picker::Item::new(format!("{} · {}", p.customer, p.name)).secondary(&p.identifier)
        })
        .collect();
    let title = format!("project for {prefix}-{description}-p…");
    let Some(&chosen) = screen.pick(&title, &rows, false)?.first() else {
        return Ok(None);
    };
    let project = &projects[chosen];

    let name = format!("{prefix}-{description}-{}", slug(&project.identifier));

    // Back to the ordinary terminal before printing: the confirmation is shell
    // output, not another panel.
    drop(screen);

    // Same shape as `wt rm`'s confirmation: what is about to happen, indented,
    // then a y/N.
    println!("about to create:");
    println!("  {name}  {} · {}", project.identifier, project.customer);
    print!("create this branch? [y/N] ");
    stdout().flush()?;
    let mut answer = String::new();
    if stdin().read_line(&mut answer)? == 0 {
        println!();
        bail!("no answer on stdin — pass the name as `wt new <words>` instead");
    }
    if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
        return Ok(None);
    }
    Ok(Some(name))
}
