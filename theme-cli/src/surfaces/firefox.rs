use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

// One theme (firefox/oxide) carries both palettes: `theme` (light) and
// `dark_theme` (dark). Firefox picks one from the system color-scheme, which
// the system surface already sets, so there is nothing to push at runtime.
// This surface only reports whether the theme is installed.
//
// Install path: Fedora's Firefox loads unsigned add-ons from the system
// extension scope (MOZ_UNSIGNED_SYSTEM_SCOPE), so firefox/install_addons.sh
// symlinks the theme there once. No web-ext, no AMO signing, no relaunch.
// See docs/oxide-theme.md and docs/theme-firefox.md.

#[cfg(target_os = "linux")]
const SYSTEM_SCOPE_LINK: &str =
    "/usr/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/oxide@theme-cli";

#[cfg(target_os = "linux")]
pub fn apply(_mode: Mode, cfg: &Config) -> SurfaceReport {
    let name = "firefox";
    let expected = cfg.repo_path.join("firefox").join("oxide");
    match std::fs::read_link(SYSTEM_SCOPE_LINK) {
        Ok(target) if target == expected => {
            SurfaceReport::ok(name, "follows system color-scheme (dark_theme)")
        }
        Ok(target) => SurfaceReport::err(
            name,
            anyhow::anyhow!(
                "{SYSTEM_SCOPE_LINK} points to {}, expected {}",
                target.display(),
                expected.display()
            ),
        ),
        Err(_) => SurfaceReport::err(
            name,
            anyhow::anyhow!("theme not installed: run firefox/install_addons.sh"),
        ),
    }
}

// macOS: Developer Edition with signatures off loads the theme as a profile
// XPI (firefox/install_addons.sh). Same dark_theme mechanism, nothing to push.
#[cfg(target_os = "macos")]
pub fn apply(_mode: Mode, _cfg: &Config) -> SurfaceReport {
    SurfaceReport::ok("firefox", "follows system color-scheme (dark_theme)")
}
