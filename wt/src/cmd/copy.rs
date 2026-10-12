use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::repo::Repo;
use crate::slug::slug;

/// Copy the current branch's slug to the clipboard — the same branch `wt open`
/// resolves, slugged like the worktree directory.
pub fn run() -> Result<()> {
    let repo = Repo::discover()?;
    let s = slug(&repo.current_branch()?);
    copy(&s)?;
    println!("==> {s}");
    Ok(())
}

/// `wl-copy` under Wayland, `xclip` otherwise (what tmux's copy mode uses).
/// Both fork to serve the selection, so stdout/stderr must not be pipes we
/// wait on.
fn copy(text: &str) -> Result<()> {
    let wayland = env::var_os("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty());
    let (prog, args): (&str, &[&str]) = if wayland {
        ("wl-copy", &[])
    } else {
        ("xclip", &["-selection", "clipboard", "-i"])
    };
    let mut child = Command::new(prog)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("running {prog} — is it installed?"))?;
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(text.as_bytes())
        .with_context(|| format!("writing to {prog}"))?;
    let status = child.wait().with_context(|| format!("waiting for {prog}"))?;
    if !status.success() {
        bail!("{prog} failed");
    }
    Ok(())
}
