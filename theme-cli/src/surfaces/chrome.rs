use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "chrome";
    let res = (|| -> Result<String> {
        let ext_id = match mode {
            Mode::Light => cfg.chrome.light_extension_id.trim(),
            Mode::Dark => cfg.chrome.dark_extension_id.trim(),
        };
        if ext_id.is_empty() {
            return Err(anyhow!(
                "chrome.{}_extension_id not set in config.toml (see chrome/README.md)",
                mode.label()
            ));
        }

        if chrome_running() {
            return Err(anyhow!(
                "Chrome is running; close it and re-run (Chrome rewrites Preferences on exit)"
            ));
        }

        let prefs_path = preferences_path()?;
        let raw = fs::read_to_string(&prefs_path)
            .with_context(|| format!("reading {}", prefs_path.display()))?;
        let mut json: Value =
            serde_json::from_str(&raw).with_context(|| format!("parsing {}", prefs_path.display()))?;

        set_theme_id(&mut json, ext_id);

        write_atomic(&prefs_path, &json)?;
        Ok(format!(
            "theme.id={} in {}",
            ext_id,
            prefs_path.display()
        ))
    })();

    match res {
        Ok(msg) => SurfaceReport::ok(name, msg),
        Err(e) => SurfaceReport::err(name, e),
    }
}

fn preferences_path() -> Result<PathBuf> {
    let cfg_dir = dirs::config_dir().ok_or_else(|| anyhow!("no XDG config dir"))?;
    let p = cfg_dir.join("google-chrome").join("Default").join("Preferences");
    if !p.exists() {
        return Err(anyhow!("Chrome Preferences not found at {}", p.display()));
    }
    Ok(p)
}

fn chrome_running() -> bool {
    for name in ["chrome", "google-chrome", "google-chrome-stable"] {
        let status = Command::new("pgrep")
            .args(["-x", name])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if let Ok(s) = status {
            if s.success() {
                return true;
            }
        }
    }
    false
}

fn set_theme_id(json: &mut Value, ext_id: &str) {
    let ext = json
        .as_object_mut()
        .expect("Preferences root is object")
        .entry("extensions")
        .or_insert_with(|| Value::Object(Default::default()));
    let theme = ext
        .as_object_mut()
        .expect("extensions is object")
        .entry("theme")
        .or_insert_with(|| Value::Object(Default::default()));
    let theme_obj = theme.as_object_mut().expect("theme is object");
    theme_obj.insert("id".into(), Value::String(ext_id.to_string()));
    theme_obj.insert("use_system".into(), Value::Bool(false));
}

fn write_atomic(path: &std::path::Path, json: &Value) -> Result<()> {
    let body = serde_json::to_vec(json).context("serializing Preferences")?;
    let dir = path.parent().ok_or_else(|| anyhow!("prefs path has no parent"))?;
    let tmp = dir.join(".Preferences.theme-cli.tmp");
    fs::write(&tmp, &body).with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}
