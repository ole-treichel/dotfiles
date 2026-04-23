use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

const IFACE: &str = "org.gnome.desktop.interface";
const SHELL_USER_THEME: &str = "org.gnome.shell.extensions.user-theme";

pub fn read() -> Result<Mode> {
    let out = Command::new("gsettings")
        .args(["get", IFACE, "color-scheme"])
        .output()
        .context("invoking gsettings get color-scheme")?;
    if !out.status.success() {
        return Err(anyhow!(
            "gsettings get color-scheme failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let trimmed = s.trim().trim_matches('\'');
    match trimmed {
        "prefer-light" => Ok(Mode::Light),
        "prefer-dark" => Ok(Mode::Dark),
        "default" => Ok(Mode::Dark),
        other => Err(anyhow!("unexpected color-scheme value: {other}")),
    }
}

pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "system";
    let res = (|| -> Result<String> {
        let theme = match mode {
            Mode::Light => &cfg.gtk.light,
            Mode::Dark => &cfg.gtk.dark,
        };
        set(IFACE, "color-scheme", mode.color_scheme())?;
        set(IFACE, "gtk-theme", theme)?;

        let shell_msg = if schema_present(SHELL_USER_THEME) {
            set(SHELL_USER_THEME, "name", theme)?;
            format!(" shell-theme={theme}")
        } else {
            String::new()
        };

        Ok(format!(
            "color-scheme={} gtk-theme={theme}{shell_msg}",
            mode.color_scheme()
        ))
    })();
    match res {
        Ok(msg) => SurfaceReport::ok(name, msg),
        Err(e) => SurfaceReport::err(name, e),
    }
}

fn set(schema: &str, key: &str, value: &str) -> Result<()> {
    let status = Command::new("gsettings")
        .args(["set", schema, key, value])
        .status()
        .with_context(|| format!("spawning gsettings set {schema} {key}"))?;
    if !status.success() {
        return Err(anyhow!("gsettings set {schema} {key} {value} failed"));
    }
    Ok(())
}

fn schema_present(schema: &str) -> bool {
    Command::new("gsettings")
        .args(["list-schemas"])
        .output()
        .ok()
        .map(|out| {
            out.status.success()
                && String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .any(|l| l.trim() == schema)
        })
        .unwrap_or(false)
}
