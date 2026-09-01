use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

const KILL_WAIT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(150);

// Firefox (stable/release) refuses to permanently install unsigned themes —
// Mozilla enforces signing outside Nightly/Dev Edition/ESR, with no override.
// The only way to load one is as a temporary add-on via `web-ext run`, which
// drives the real profile over the remote debugger protocol. That requires
// `--keep-profile-changes`, which Mozilla's own docs call destructive and
// insecure for daily use (it enables silent remote debugging connections,
// among other things) — accepted trade-off for using the real profile here.

// ---------------------------------------------------------------------------
// Linux: Flatpak-based Firefox, driven through `web-ext run`
// ---------------------------------------------------------------------------
#[cfg(target_os = "linux")]
pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "firefox";
    let res = (|| -> Result<String> {
        let app_id = cfg.firefox.flatpak_app_id.trim();
        if app_id.is_empty() {
            return Ok("disabled".to_string());
        }

        let ext_dir = cfg.repo_path.join("firefox").join(match mode {
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

        launch_web_ext(&ext_dir, &format!("flatpak:{app_id}"), &cfg.firefox.profile)?;

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

// ---------------------------------------------------------------------------
// macOS: native app, driven through `web-ext run`
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
pub fn apply(mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "firefox";
    let res = (|| -> Result<String> {
        let app_name = cfg.firefox.app_name.trim();
        if app_name.is_empty() {
            return Ok("disabled".to_string());
        }

        let ext_dir = cfg.repo_path.join("firefox").join(match mode {
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

        launch_web_ext(&ext_dir, "firefox", &cfg.firefox.profile)?;

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

// ---------------------------------------------------------------------------
// Shared: spawn `web-ext run`, detached, loading the theme as a temporary
// add-on into the given Firefox target and profile.
// ---------------------------------------------------------------------------
fn launch_web_ext(ext_dir: &std::path::Path, firefox_target: &str, profile: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = Command::new("setsid");
        c.args(["--fork", "npx", "--yes", "web-ext", "run"]);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = Command::new("npx");
    #[cfg(target_os = "macos")]
    cmd.args(["--yes", "web-ext", "run"]);

    cmd.arg("--source-dir")
        .arg(ext_dir)
        .arg("--firefox")
        .arg(firefox_target)
        .arg("--firefox-profile")
        .arg(profile)
        .arg("--keep-profile-changes")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    cmd.spawn()
        .with_context(|| format!("spawning web-ext run for {}", ext_dir.display()))?;
    Ok(())
}
