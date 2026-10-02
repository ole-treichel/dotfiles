use std::io::{stdin, stdout, Write};

use anyhow::{bail, Result};

use crate::hooks::{self, HookCtx};
use crate::picker;
use crate::repo::Repo;
use crate::slug::slug;
use crate::{git, moco, tms};

/// `feat`, `chore`, `fix`, `scratch` and nothing else, with what each one is
/// for.
const PREFIXES: [(&str, &str); 4] = [
    ("feat", "new feature"),
    ("chore", "maintenance"),
    ("fix", "bugfix"),
    ("scratch", "experiment, project optional"),
];

/// The one prefix whose project number is optional.
const SCRATCH: &str = "scratch";

/// Create a branch + worktree from the base ref, then run the post-create hooks.
///
/// No words: the wizard walks prefix → description → MOCO project and builds
/// `<prefix>-<description>-p<number>`; for `scratch` the project, and with it
/// the number, is optional. Words: they are slugged and used as given — no
/// prefix, no project number.
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
    let optional = prefix == SCRATCH;
    let number = if optional { "[-p…]" } else { "-p…" };

    // Loops rather than erroring: a stray enter should redraw the step, not
    // end the run. `esc` is the way out, same as in every picker.
    let description = loop {
        let title = format!("{prefix}-<description>{number}");
        let Some(typed) = screen.prompt(&title, "type the description")? else {
            return Ok(None);
        };
        let slugged = slug(&typed);
        if !slugged.is_empty() {
            break slugged;
        }
    };

    // A scratch branch does without a number, so an unreachable MOCO only
    // costs it the picker. The reason is shown in the confirmation.
    let mut unreachable = None;
    let projects = match moco::projects() {
        Ok(projects) => projects,
        Err(e) if optional => {
            unreachable = Some(e);
            Vec::new()
        }
        Err(e) => bail!("{e:#} — `{SCRATCH}` branches need no project"),
    };

    let project = if unreachable.is_some() {
        None
    } else {
        // Scratch: "no project" leads the list, so enter on an untouched
        // filter takes the common case.
        let skip = usize::from(optional);
        let mut rows: Vec<picker::Item> = Vec::with_capacity(projects.len() + skip);
        if optional {
            rows.push(picker::Item::new("no project").secondary(format!("{prefix}-{description}")));
        }
        rows.extend(projects.iter().map(|p| {
            // Customer first: it is the coarser sort, so the left edge of the
            // column groups the list by client as you scan it.
            picker::Item::new(format!("{} · {}", p.customer, p.name)).secondary(&p.identifier)
        }));
        let title = format!("project for {prefix}-{description}{number}");
        let Some(&chosen) = screen.pick(&title, &rows, false)?.first() else {
            return Ok(None);
        };
        chosen.checked_sub(skip).map(|i| &projects[i])
    };

    let name = branch_name(prefix, &description, project.map(|p| p.identifier.as_str()));

    // Back to the ordinary terminal before printing: the confirmation is shell
    // output, not another panel.
    drop(screen);

    let detail = match (project, &unreachable) {
        (Some(p), _) => format!("{} · {}", p.identifier, p.customer),
        (None, Some(e)) => format!("no project ({e:#})"),
        (None, None) => "no project".to_string(),
    };

    // Same shape as `wt rm`'s confirmation: what is about to happen, indented,
    // then a y/N.
    println!("about to create:");
    println!("  {name}  {detail}");
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

/// `<prefix>-<description>-p<number>`, or without the number when there is no
/// project. `description` is already slugged.
fn branch_name(prefix: &str, description: &str, identifier: Option<&str>) -> String {
    match identifier {
        Some(id) => format!("{prefix}-{description}-{}", slug(id)),
        None => format!("{prefix}-{description}"),
    }
}

#[cfg(test)]
mod tests {
    use super::branch_name;

    #[test]
    fn the_number_goes_last_and_lowercase() {
        assert_eq!(
            branch_name("feat", "import-button", Some("P26059")),
            "feat-import-button-p26059"
        );
        assert_eq!(
            branch_name("fix", "x", Some("P1903-003")),
            "fix-x-p1903-003"
        );
    }

    #[test]
    fn scratch_without_a_project_has_no_number() {
        assert_eq!(branch_name("scratch", "cache-poc", None), "scratch-cache-poc");
        assert_eq!(
            branch_name("scratch", "cache-poc", Some("P26059")),
            "scratch-cache-poc-p26059"
        );
    }
}
