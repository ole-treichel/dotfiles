use std::fs::File;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "gnome-terminal";
    let res = (|| -> Result<String> {
        let uuid = cfg.gnome_terminal.profile_uuid.trim();
        if uuid.is_empty() {
            return Err(anyhow!(
                "gnome_terminal.profile_uuid not set in config.toml (see gnome-terminal/README.md)"
            ));
        }

        let file = cfg.repo_path.join("gnome-terminal").join(match mode {
            Mode::Light => "everforest-light.dconf",
            Mode::Dark => "everforest-dark.dconf",
        });
        if !file.exists() {
            return Err(anyhow!("dconf file not found: {}", file.display()));
        }

        let dconf_path = format!("/org/gnome/terminal/legacy/profiles:/:{uuid}/");
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
        Ok(format!("loaded {} into {dconf_path}", file.display()))
    })();

    match res {
        Ok(msg) => SurfaceReport::ok(name, msg),
        Err(e) => SurfaceReport::err(name, e),
    }
}
