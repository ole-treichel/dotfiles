use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

// ---------------------------------------------------------------------------
// Linux: gsettings + GTK
// ---------------------------------------------------------------------------
#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};

    const IFACE: &str = "org.gnome.desktop.interface";
    const SHELL_USER_THEME: &str = "org.gnome.shell.extensions.user-theme";
    const GTK4_CSS_FILES: &[&str] = &["assets", "gtk.css", "gtk-dark.css"];

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
            let gtk = cfg
                .gtk
                .as_ref()
                .ok_or_else(|| anyhow!("[gtk] section missing from config"))?;
            let theme = match mode {
                Mode::Light => &gtk.light,
                Mode::Dark => &gtk.dark,
            };

            let swapped = swap_gtk4_symlinks(theme)?;

            set(IFACE, "color-scheme", mode.color_scheme())?;
            set(IFACE, "gtk-theme", theme)?;

            let shell_msg = if schema_present(SHELL_USER_THEME) {
                set(SHELL_USER_THEME, "name", theme)?;
                format!(" shell-theme={theme}")
            } else {
                String::new()
            };

            let symlink_msg = if swapped > 0 {
                format!(" gtk-4.0-symlinks={swapped}")
            } else {
                String::new()
            };

            Ok(format!(
                "color-scheme={} gtk-theme={theme}{shell_msg}{symlink_msg}",
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

    fn swap_gtk4_symlinks(theme: &str) -> Result<usize> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("no home dir"))?;
        let theme_dir = home.join(".themes").join(theme).join("gtk-4.0");
        let config_dir = home.join(".config").join("gtk-4.0");

        if !theme_dir.exists() {
            return Ok(0);
        }
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("creating {}", config_dir.display()))?;
        }

        let mut count = 0usize;
        for name in GTK4_CSS_FILES {
            let target = theme_dir.join(name);
            if !target.exists() {
                continue;
            }
            let link = config_dir.join(name);
            if link.exists() && !link.is_symlink() {
                continue;
            }
            retarget_symlink(&target, &link).with_context(|| {
                format!("retargeting {} -> {}", link.display(), target.display())
            })?;
            count += 1;
        }
        Ok(count)
    }

    fn retarget_symlink(target: &Path, link: &Path) -> Result<()> {
        let tmp = tmp_path(link);
        let _ = fs::remove_file(&tmp);
        symlink(target, &tmp).with_context(|| format!("symlinking {}", tmp.display()))?;
        fs::rename(&tmp, link)
            .with_context(|| format!("renaming {} -> {}", tmp.display(), link.display()))?;
        Ok(())
    }

    fn tmp_path(link: &Path) -> PathBuf {
        let fname = link
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let parent = link.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!(".{fname}.theme-cli.tmp"))
    }
}

// ---------------------------------------------------------------------------
// macOS: defaults + osascript
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    pub fn read() -> Result<Mode> {
        let output = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
            .context("invoking defaults read -g AppleInterfaceStyle")?;
        if !output.status.success() {
            return Ok(Mode::Light);
        }
        let val = String::from_utf8_lossy(&output.stdout);
        if val.trim().eq_ignore_ascii_case("dark") {
            Ok(Mode::Dark)
        } else {
            Ok(Mode::Light)
        }
    }

    pub fn apply(mode: Mode, _cfg: &Config) -> SurfaceReport {
        let name = "system";
        let dark_mode = mode == Mode::Dark;
        let script = format!(
            "tell application \"System Events\" to tell appearance preferences to set dark mode to {}",
            dark_mode
        );
        let res = Command::new("osascript")
            .args(["-e", &script])
            .status()
            .context("osascript set dark mode");
        match res {
            Ok(s) if s.success() => SurfaceReport::ok(name, format!("dark-mode={dark_mode}")),
            Ok(s) => SurfaceReport::err(name, anyhow!("osascript exit {s}")),
            Err(e) => SurfaceReport::err(name, e),
        }
    }
}

// ---------------------------------------------------------------------------
// Public API — delegates to the platform module
// ---------------------------------------------------------------------------
#[cfg(target_os = "linux")]
pub use linux::{apply, read};

#[cfg(target_os = "macos")]
pub use macos::{apply, read};
