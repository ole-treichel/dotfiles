use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

const SCHEMA: &str = "org.gnome.desktop.interface";

pub fn read() -> Result<Mode> {
    let out = Command::new("gsettings")
        .args(["get", SCHEMA, "color-scheme"])
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
        "default" => Ok(Mode::Dark), // treat as dark when unset
        other => Err(anyhow!("unexpected color-scheme value: {other}")),
    }
}

pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "system";
    let res = (|| -> Result<String> {
        set(SCHEMA, "color-scheme", mode.color_scheme())?;
        let gtk_theme = match mode {
            Mode::Light => &cfg.gtk.light,
            Mode::Dark => &cfg.gtk.dark,
        };
        set(SCHEMA, "gtk-theme", gtk_theme)?;
        Ok(format!(
            "color-scheme={} gtk-theme={}",
            mode.color_scheme(),
            gtk_theme
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
