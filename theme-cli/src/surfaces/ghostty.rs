use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "ghostty";
    let res = (|| -> Result<String> {
        let ghostty_cfg = cfg
            .ghostty
            .as_ref()
            .ok_or_else(|| anyhow!("[ghostty] section missing from config"))?;

        let theme_name = match mode {
            Mode::Light => &ghostty_cfg.light,
            Mode::Dark => &ghostty_cfg.dark,
        };

        let config_path = resolve_config_path(ghostty_cfg)?;
        rewrite_theme_line(&config_path, theme_name)?;
        trigger_reload();
        Ok(format!("theme={theme_name}"))
    })();
    match res {
        Ok(msg) => SurfaceReport::ok(name, msg),
        Err(e) => SurfaceReport::err(name, e),
    }
}

/// Nudge Ghostty to reload its config by clicking the menu item via AppleScript.
fn trigger_reload() {
    let _ = Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events"
    if exists (process "ghostty") then
        tell process "ghostty"
            click menu item "Reload Configuration" of menu 1 of menu bar item "Ghostty" of menu bar 1
        end tell
    end if
end tell"#,
        ])
        .output();
}

fn resolve_config_path(ghostty_cfg: &crate::config::Ghostty) -> Result<PathBuf> {
    if !ghostty_cfg.config_path.is_empty() {
        return Ok(PathBuf::from(&ghostty_cfg.config_path));
    }

    // XDG path (works if user symlinks)
    let xdg = dirs::home_dir()
        .map(|h| h.join(".config").join("ghostty").join("config"));
    if let Some(ref p) = xdg {
        if p.exists() {
            return Ok(p.clone());
        }
    }

    // macOS native path
    let native = dirs::config_dir()
        .map(|c| c.join("com.mitchellh.ghostty").join("config"));
    if let Some(ref p) = native {
        if p.exists() {
            return Ok(p.clone());
        }
    }

    // Fall back to native path even if it doesn't exist yet
    native.ok_or_else(|| anyhow!("cannot determine ghostty config path"))
}

fn rewrite_theme_line(config_path: &PathBuf, new_theme: &str) -> Result<()> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("reading {}", config_path.display()))?;

    let mut found = false;
    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("theme") {
                if let Some(rest) = trimmed.strip_prefix("theme") {
                    let rest = rest.trim_start();
                    if rest.starts_with('=') {
                        found = true;
                        return format!("theme = {new_theme}");
                    }
                }
            }
            line.to_string()
        })
        .collect();

    let final_content = if found {
        let mut s = new_lines.join("\n");
        if content.ends_with('\n') {
            s.push('\n');
        }
        s
    } else {
        format!("theme = {new_theme}\n{content}")
    };

    fs::write(config_path, final_content)
        .with_context(|| format!("writing {}", config_path.display()))?;
    Ok(())
}
