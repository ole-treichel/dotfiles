use std::process::Command;

use anyhow::{Context, Result};
use glob::glob;

use crate::config::Config;
use crate::surfaces::{Mode, SurfaceReport};

pub fn apply(mode: Mode, _cfg: &Config) -> SurfaceReport {
    let name = "nvim";
    match push_to_running(mode) {
        Ok(count) => SurfaceReport::ok(name, format!("pushed to {count} running server(s)")),
        Err(e) => SurfaceReport::err(name, e),
    }
}

fn push_to_running(mode: Mode) -> Result<usize> {
    let sockets = find_sockets()?;
    let cmd = format!("<Esc>:set background={}<CR>", mode.label());
    let mut ok = 0usize;
    for sock in sockets {
        let status = Command::new("nvim")
            .args(["--server"])
            .arg(&sock)
            .args(["--remote-send", &cmd])
            .status()
            .with_context(|| format!("invoking nvim --server {}", sock.display()))?;
        if status.success() {
            ok += 1;
        } else {
            eprintln!("nvim: server at {} did not accept the command", sock.display());
        }
    }
    Ok(ok)
}

fn find_sockets() -> Result<Vec<std::path::PathBuf>> {
    let mut found = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let mut roots = Vec::new();
        if let Some(rt) = std::env::var_os("XDG_RUNTIME_DIR") {
            roots.push(std::path::PathBuf::from(rt));
        }
        roots.push(std::path::PathBuf::from("/tmp"));

        for root in roots {
            let pattern = root.join("nvim.*.0");
            let pat = pattern.to_string_lossy().to_string();
            for entry in glob(&pat).with_context(|| format!("glob {pat}"))?.flatten() {
                found.push(entry);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS nvim sockets: $TMPDIR/nvim.<user>/<hash>/nvim.<pid>.0
        if let Some(tmpdir) = std::env::var_os("TMPDIR") {
            let pattern = std::path::PathBuf::from(tmpdir)
                .join("nvim.*/*/nvim.*.0");
            let pat = pattern.to_string_lossy().to_string();
            for entry in glob(&pat).with_context(|| format!("glob {pat}"))?.flatten() {
                found.push(entry);
            }
        }
        // Also check /tmp as fallback
        let pat = "/tmp/nvim.*/*/nvim.*.0";
        for entry in glob(pat).with_context(|| format!("glob {pat}"))?.flatten() {
            found.push(entry);
        }
    }

    Ok(found)
}
