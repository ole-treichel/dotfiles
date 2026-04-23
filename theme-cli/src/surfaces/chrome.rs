use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

const KILL_WAIT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(150);

// ---------------------------------------------------------------------------
// Linux: Flatpak-based Chrome
// ---------------------------------------------------------------------------
#[cfg(target_os = "linux")]
pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "chrome";
    let res = (|| -> Result<String> {
        let app_id = cfg.chrome.flatpak_app_id.trim();
        if app_id.is_empty() {
            return Ok("disabled".to_string());
        }

        let ext_dir = cfg.repo_path.join("chrome").join(match mode {
            Mode::Light => "everforest-light",
            Mode::Dark => "everforest-dark",
        });
        if !ext_dir.exists() {
            return Err(anyhow!("extension dir not found: {}", ext_dir.display()));
        }

        ensure_filesystem_access(app_id, &cfg.repo_path)?;

        let was_running = flatpak_running(app_id)?;
        if was_running {
            flatpak_kill(app_id)?;
            wait_for_exit(app_id, KILL_WAIT)?;
        }

        launch_flatpak(app_id, &ext_dir)?;

        Ok(if was_running {
            format!("relaunched with {}", ext_dir.display())
        } else {
            format!("launched with {}", ext_dir.display())
        })
    })();

    match res {
        Ok(msg) => SurfaceReport::ok(name, msg),
        Err(e) => SurfaceReport::err(name, e),
    }
}

#[cfg(target_os = "linux")]
fn ensure_filesystem_access(app_id: &str, repo_path: &std::path::Path) -> Result<()> {
    let arg = format!("--filesystem={}:ro", repo_path.display());
    let status = Command::new("flatpak")
        .args(["override", "--user", &arg, app_id])
        .status()
        .context("flatpak override")?;
    if !status.success() {
        return Err(anyhow!("flatpak override for {app_id} failed"));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn flatpak_running(app_id: &str) -> Result<bool> {
    let out = Command::new("flatpak")
        .args(["ps", "--columns=application"])
        .output()
        .context("flatpak ps")?;
    if !out.status.success() {
        return Err(anyhow!("flatpak ps failed"));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|l| l.trim() == app_id))
}

#[cfg(target_os = "linux")]
fn flatpak_kill(app_id: &str) -> Result<()> {
    let status = Command::new("flatpak")
        .args(["kill", app_id])
        .status()
        .context("flatpak kill")?;
    if !status.success() {
        return Err(anyhow!("flatpak kill {app_id} failed"));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn wait_for_exit(app_id: &str, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !flatpak_running(app_id)? {
            return Ok(());
        }
        thread::sleep(POLL);
    }
    Err(anyhow!(
        "{app_id} still running after {}s",
        timeout.as_secs()
    ))
}

#[cfg(target_os = "linux")]
fn launch_flatpak(app_id: &str, ext_dir: &std::path::Path) -> Result<()> {
    let ext_arg = format!("--load-extension={}", ext_dir.display());
    Command::new("setsid")
        .args(["--fork", "flatpak", "run", app_id])
        .arg(&ext_arg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawning flatpak run {app_id}"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// macOS: regular app via `open -a`
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "chrome";
    let res = (|| -> Result<String> {
        let app_name = cfg.chrome.app_name.trim();
        if app_name.is_empty() {
            return Ok("disabled".to_string());
        }

        let ext_dir = cfg.repo_path.join("chrome").join(match mode {
            Mode::Light => "everforest-light",
            Mode::Dark => "everforest-dark",
        });
        if !ext_dir.exists() {
            return Err(anyhow!("extension dir not found: {}", ext_dir.display()));
        }

        let was_running = process_running(app_name)?;
        if was_running {
            quit_app(app_name)?;
            wait_for_process_exit(app_name, KILL_WAIT)?;
        }

        launch_app(app_name, &ext_dir)?;

        Ok(if was_running {
            format!("relaunched with {}", ext_dir.display())
        } else {
            format!("launched with {}", ext_dir.display())
        })
    })();

    match res {
        Ok(msg) => SurfaceReport::ok(name, msg),
        Err(e) => SurfaceReport::err(name, e),
    }
}

#[cfg(target_os = "macos")]
fn process_running(app_name: &str) -> Result<bool> {
    let status = Command::new("pgrep")
        .args(["-xi", app_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("pgrep -xi {app_name}"))?;
    Ok(status.success())
}

#[cfg(target_os = "macos")]
fn quit_app(app_name: &str) -> Result<()> {
    let script = format!("tell application \"{app_name}\" to quit");
    Command::new("osascript")
        .args(["-e", &script])
        .status()
        .with_context(|| format!("osascript quit {app_name}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn wait_for_process_exit(app_name: &str, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !process_running(app_name)? {
            return Ok(());
        }
        thread::sleep(POLL);
    }
    Err(anyhow!(
        "{app_name} still running after {}s",
        timeout.as_secs()
    ))
}

#[cfg(target_os = "macos")]
fn launch_app(app_name: &str, ext_dir: &std::path::Path) -> Result<()> {
    let ext_arg = format!("--load-extension={}", ext_dir.display());
    Command::new("open")
        .args(["-a", app_name, "--args", &ext_arg])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("opening {app_name}"))?;
    Ok(())
}
