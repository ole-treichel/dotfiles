use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

fn cmd(dir: &Path, args: &[&str]) -> Command {
    let mut c = Command::new("git");
    c.arg("-C").arg(dir).args(args);
    c
}

/// Run git, capture stdout, error out with git's stderr on a non-zero exit.
pub fn out(dir: &Path, args: &[&str]) -> Result<String> {
    let output = cmd(dir, args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
}

/// Run git, let it write to the terminal, error out on a non-zero exit.
pub fn run(dir: &Path, args: &[&str]) -> Result<()> {
    let status = cmd(dir, args)
        .stdin(Stdio::null())
        .status()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !status.success() {
        bail!("git {} failed", args.join(" "));
    }
    Ok(())
}

/// Run git quietly, returning whether it succeeded. For probes.
pub fn ok(dir: &Path, args: &[&str]) -> bool {
    cmd(dir, args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run git, capturing stderr so it can be reported by the caller.
pub fn try_run(dir: &Path, args: &[&str]) -> Result<Result<(), String>> {
    let output = cmd(dir, args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if output.status.success() {
        Ok(Ok(()))
    } else {
        let mut msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if msg.is_empty() {
            msg = format!("git {} failed", args.join(" "));
        }
        Ok(Err(msg))
    }
}
