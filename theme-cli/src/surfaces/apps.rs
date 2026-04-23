use std::process::Command;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

/// Processes that cache GTK4 CSS on startup and need a fresh launch
/// to pick up theme changes. `pkill -x` matches the process name exactly;
/// each entry also catches background D-Bus-activated service variants
/// because they run with the same argv[0].
const STUBBORN: &[&str] = &["nautilus"];

pub fn apply(_mode: Mode, _cfg: &Config) -> SurfaceReport {
    let name = "apps";
    match kill_stubborn() {
        Ok(killed) if killed.is_empty() => SurfaceReport::ok(name, "nothing to restart"),
        Ok(killed) => SurfaceReport::ok(name, format!("killed: {}", killed.join(", "))),
        Err(e) => SurfaceReport::err(name, e),
    }
}

fn kill_stubborn() -> Result<Vec<String>> {
    let mut killed = Vec::new();
    for proc in STUBBORN {
        if pgrep(proc)? {
            Command::new("pkill")
                .args(["-x", proc])
                .status()
                .with_context(|| format!("pkill -x {proc}"))?;
            killed.push((*proc).to_string());
        }
    }
    Ok(killed)
}

fn pgrep(name: &str) -> Result<bool> {
    let status = Command::new("pgrep")
        .args(["-x", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| format!("pgrep -x {name}"))?;
    Ok(status.success())
}
