use std::fs::File;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

/// Fixed UUID for the Everforest profile. Baked in so the CLI can create
/// and reference the profile without any manual setup in GNOME Terminal.
const PROFILE_UUID: &str = "3b1a4e2f-8c7d-4b9e-a1f2-c3d4e5f6a7b8";
const PROFILE_NAME: &str = "Everforest";

pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "gnome-terminal";
    let res = (|| -> Result<String> {
        let created = ensure_profile_registered()?;

        let file = cfg.repo_path.join("gnome-terminal").join(match mode {
            Mode::Light => "everforest-light.dconf",
            Mode::Dark => "everforest-dark.dconf",
        });
        if !file.exists() {
            return Err(anyhow!("dconf file not found: {}", file.display()));
        }

        let dconf_path = format!("/org/gnome/terminal/legacy/profiles:/:{PROFILE_UUID}/");
        let input = File::open(&file)
            .with_context(|| format!("opening {}", file.display()))?;
        let status = Command::new("dconf")
            .args(["load", &dconf_path])
            .stdin(Stdio::from(input))
            .status()
            .with_context(|| format!("dconf load {dconf_path}"))?;
        if !status.success() {
            return Err(anyhow!("dconf load {dconf_path} failed"));
        }

        Ok(if created {
            format!("created profile '{PROFILE_NAME}' and loaded {}", file.display())
        } else {
            format!("loaded {}", file.display())
        })
    })();

    match res {
        Ok(msg) => SurfaceReport::ok(name, msg),
        Err(e) => SurfaceReport::err(name, e),
    }
}

/// Ensures the Everforest profile is registered in the terminal's profile
/// list. Returns `true` if we just created it, `false` if it already existed.
fn ensure_profile_registered() -> Result<bool> {
    let current = dconf_read("/org/gnome/terminal/legacy/profiles:/list")?;
    let mut uuids = parse_string_list(&current);
    if uuids.iter().any(|u| u == PROFILE_UUID) {
        return Ok(false);
    }

    uuids.push(PROFILE_UUID.to_string());
    let new_list = format_string_list(&uuids);
    dconf_write("/org/gnome/terminal/legacy/profiles:/list", &new_list)?;

    let name_key = format!("/org/gnome/terminal/legacy/profiles:/:{PROFILE_UUID}/visible-name");
    dconf_write(&name_key, &format!("'{PROFILE_NAME}'"))?;

    // First-time registration: make it the default so new terminals open with it.
    let default_key = "/org/gnome/terminal/legacy/profiles:/default";
    dconf_write(default_key, &format!("'{PROFILE_UUID}'"))?;

    Ok(true)
}

fn dconf_read(path: &str) -> Result<String> {
    let out = Command::new("dconf")
        .args(["read", path])
        .output()
        .with_context(|| format!("dconf read {path}"))?;
    if !out.status.success() {
        return Err(anyhow!(
            "dconf read {path} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn dconf_write(path: &str, value: &str) -> Result<()> {
    let status = Command::new("dconf")
        .args(["write", path, value])
        .status()
        .with_context(|| format!("dconf write {path}"))?;
    if !status.success() {
        return Err(anyhow!("dconf write {path} {value} failed"));
    }
    Ok(())
}

/// Parse a GVariant string array, e.g. `['abc', 'def']` or `@as []`.
fn parse_string_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "@as []" {
        return Vec::new();
    }
    trimmed
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().trim_matches('\'').trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn format_string_list(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|s| format!("'{s}'")).collect();
    format!("[{}]", inner.join(", "))
}
