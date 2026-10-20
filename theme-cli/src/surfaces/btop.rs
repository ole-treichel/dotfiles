use std::fs;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

/// btop saves its in-memory config on exit, so editing btop.conf alone would be
/// reverted by a running instance. SIGUSR2 makes btop reload btop.conf first.
pub fn apply(mode: Mode, _cfg: &Config) -> SurfaceReport {
    let name = "btop";
    let theme = match mode {
        Mode::Light => "oxide-light",
        Mode::Dark => "oxide-dark",
    };
    match rewrite_theme(theme) {
        Ok(Some(msg)) => SurfaceReport::ok(name, msg),
        Ok(None) => SurfaceReport::ok(name, "no btop.conf"),
        Err(e) => SurfaceReport::err(name, e),
    }
}

fn rewrite_theme(theme: &str) -> Result<Option<String>> {
    // btop reads ~/.config on macOS too, not ~/Library/Application Support.
    let path = dirs::home_dir()
        .ok_or_else(|| anyhow!("no home dir"))?
        .join(".config/btop/btop.conf");
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(None);
    };
    fs::write(&path, set_theme(&content, theme)).with_context(|| format!("writing {}", path.display()))?;

    // Exit code 1 means no btop running; the file edit is picked up on next start.
    let status = Command::new("pkill").args(["-USR2", "-x", "btop"]).status()?;
    if !status.success() && status.code() != Some(1) {
        return Err(anyhow!("pkill -USR2 btop exit {status}"));
    }
    Ok(Some(format!("color_theme={theme} ({})", path.display())))
}

fn set_theme(content: &str, theme: &str) -> String {
    let line = format!("color_theme = \"{theme}\"");
    let mut found = false;
    let mut out: Vec<&str> = content
        .lines()
        .map(|l| {
            if l.trim_start().starts_with("color_theme") {
                found = true;
                line.as_str()
            } else {
                l
            }
        })
        .collect();
    if !found {
        out.push(&line);
    }
    out.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::set_theme;

    #[test]
    fn replaces_or_appends_color_theme() {
        let conf = "#* color_theme comment\ncolor_theme = \"/usr/share/btop/themes/x.theme\"\ntruecolor = true\n";
        assert_eq!(
            set_theme(conf, "oxide-dark"),
            "#* color_theme comment\ncolor_theme = \"oxide-dark\"\ntruecolor = true\n"
        );
        assert_eq!(set_theme("truecolor = true\n", "oxide-light"), "truecolor = true\ncolor_theme = \"oxide-light\"\n");
    }
}
