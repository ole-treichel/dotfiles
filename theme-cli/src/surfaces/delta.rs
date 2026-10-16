use std::process::Command;

use anyhow::{anyhow, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

/// delta has no live light/dark detection when used as `interactive.diffFilter`
/// (it can't safely query the terminal through the pipe), so we push the
/// mode into ~/.gitconfig explicitly, same as every other surface.
/// `delta.features` selects the `[delta "oxide-*"]` section in .gitconfig
/// (syntax theme and diff colors).
pub fn apply(mode: Mode, _cfg: &Config) -> SurfaceReport {
    let name = "delta";
    let (set_key, unset_key) = match mode {
        Mode::Light => ("light", "dark"),
        Mode::Dark => ("dark", "light"),
    };
    let feature = match mode {
        Mode::Light => "oxide-light",
        Mode::Dark => "oxide-dark",
    };
    match set_and_unset(set_key, unset_key).and_then(|()| git_config_set("delta.features", feature)) {
        Ok(()) => SurfaceReport::ok(name, format!("delta.{set_key}=true features={feature}")),
        Err(e) => SurfaceReport::err(name, e),
    }
}

fn git_config_set(key: &str, value: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["config", "--global", key, value])
        .status()?;
    if !status.success() {
        return Err(anyhow!("git config --global {key} {value} failed"));
    }
    Ok(())
}

fn set_and_unset(set_key: &str, unset_key: &str) -> Result<()> {
    git_config_set(&format!("delta.{set_key}"), "true")?;

    // Exit code 5 means the key wasn't set; both are fine outcomes here.
    let status = Command::new("git")
        .args(["config", "--global", "--unset", &format!("delta.{unset_key}")])
        .status()?;
    if !status.success() && status.code() != Some(5) {
        return Err(anyhow!("git config --global --unset delta.{unset_key} failed"));
    }
    Ok(())
}
