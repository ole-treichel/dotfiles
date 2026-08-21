use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

pub fn apply(mode: Mode, _cfg: &Config) -> SurfaceReport {
    let name = "hunk";
    let theme = match mode {
        Mode::Light => "everforest-light",
        Mode::Dark => "everforest-dark",
    };
    match rewrite_theme(theme) {
        Ok(path) => SurfaceReport::ok(name, format!("theme={theme} ({})", path.display())),
        Err(e) => SurfaceReport::err(name, e),
    }
}

fn config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow!("no config dir"))?;
    Ok(base.join("hunk").join("config.toml"))
}

fn rewrite_theme(theme: &str) -> Result<PathBuf> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }

    let content = fs::read_to_string(&path).unwrap_or_default();
    let mut found = false;
    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("theme") {
                let rest = rest.trim_start();
                if rest.starts_with('=') {
                    found = true;
                    return format!("theme = \"{theme}\"");
                }
            }
            line.to_string()
        })
        .collect();

    let final_content = if found {
        let mut s = new_lines.join("\n");
        s.push('\n');
        s
    } else {
        format!("theme = \"{theme}\"\n{content}")
    };

    fs::write(&path, final_content).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}
