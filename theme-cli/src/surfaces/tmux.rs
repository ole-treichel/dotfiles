use std::process::Command;

use anyhow::{anyhow, Context};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "tmux";
    let theme_file = cfg.repo_path.join("tmux").join(match mode {
        Mode::Light => "everforest-light.conf",
        Mode::Dark => "everforest-dark.conf",
    });

    if !theme_file.exists() {
        return SurfaceReport::err(
            name,
            anyhow!("theme file not found: {}", theme_file.display()),
        );
    }

    if !server_running() {
        return SurfaceReport::ok(name, "no server running");
    }

    let path_str = theme_file.to_string_lossy();
    let res = Command::new("tmux")
        .args(["source-file", &path_str])
        .status()
        .with_context(|| format!("tmux source-file {path_str}"));

    match res {
        Ok(status) if status.success() => SurfaceReport::ok(name, format!("sourced {}", theme_file.display())),
        Ok(status) => SurfaceReport::err(name, anyhow!("tmux source-file exit {status}")),
        Err(e) => SurfaceReport::err(name, e),
    }
}

fn server_running() -> bool {
    Command::new("tmux")
        .arg("list-sessions")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
