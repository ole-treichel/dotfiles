use anyhow::Result;

use crate::repo::Repo;

/// Table of worktrees: directory, branch, HEAD, state.
pub fn run() -> Result<()> {
    let repo = Repo::discover()?;
    let checkouts = repo.checkouts()?;
    if checkouts.is_empty() {
        println!("no worktrees in {}", repo.root.display());
        return Ok(());
    }
    let default_branch = repo.default_branch().unwrap_or_default();

    let rows: Vec<[String; 4]> = checkouts
        .iter()
        .map(|w| {
            let branch = w.branch.clone().unwrap_or_else(|| "(detached)".into());
            let head = w.head.as_deref().unwrap_or("").chars().take(7).collect();
            let mut state = Vec::new();
            if w.prunable {
                state.push("prunable".to_string());
            } else {
                if w.dirty() {
                    state.push("dirty".to_string());
                }
                let ahead = w.unpushed(&repo.root);
                if ahead > 0 {
                    state.push(format!("↑{ahead}"));
                }
                if w.locked {
                    state.push("locked".to_string());
                }
            }
            if branch == default_branch {
                state.push("default".to_string());
            }
            [w.name(), branch, head, state.join(" ")]
        })
        .collect();

    let header = ["DIR", "BRANCH", "HEAD", "STATE"];
    let mut widths = header.map(str::len);
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }
    let line = |cells: [&str; 4]| {
        let mut s = String::new();
        for (i, cell) in cells.iter().enumerate() {
            if i == cells.len() - 1 {
                s.push_str(cell);
            } else {
                s.push_str(&format!("{:<width$}  ", cell, width = widths[i]));
            }
        }
        s.trim_end().to_string()
    };

    println!("{}", line(header));
    for row in &rows {
        println!("{}", line([&row[0], &row[1], &row[2], &row[3]]));
    }
    Ok(())
}
