use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub repo_path: PathBuf,
    pub gtk: Gtk,
    pub gnome_terminal: GnomeTerminal,
}

#[derive(Debug, Deserialize)]
pub struct Gtk {
    pub light: String,
    pub dark: String,
}

#[derive(Debug, Deserialize)]
pub struct GnomeTerminal {
    pub profile_uuid: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::resolve_path()?;
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("reading config from {}", path.display()))?;
        let cfg: Config =
            toml::from_str(&raw).with_context(|| format!("parsing config {}", path.display()))?;
        Ok(cfg)
    }

    fn resolve_path() -> Result<PathBuf> {
        if let Some(base) = dirs::config_dir() {
            let user = base.join("theme-cli").join("config.toml");
            if user.exists() {
                return Ok(user);
            }
        }
        let crate_cfg = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.toml");
        Ok(crate_cfg)
    }
}
