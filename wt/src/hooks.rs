use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

pub struct HookCtx {
    pub event: &'static str,
    pub repo_root: PathBuf,
    pub dir: PathBuf,
    pub branch: String,
    pub slug: String,
    pub base_ref: String,
    pub default_branch: String,
}

/// `wt/hooks/post-create.d`, resolved from the binary's own location so the
/// symlink in `~/.local/bin` still finds the dropins in this repo.
fn hooks_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("WT_HOOKS_DIR") {
        return Some(PathBuf::from(dir));
    }
    let exe = std::env::current_exe().ok()?;
    let exe = fs::canonicalize(&exe).unwrap_or(exe);
    // <crate>/target/<profile>/wt
    let from_exe = exe
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(|crate_dir| crate_dir.join("hooks/post-create.d"));
    match from_exe {
        Some(d) if d.is_dir() => Some(d),
        _ => {
            let d = Path::new(env!("CARGO_MANIFEST_DIR")).join("hooks/post-create.d");
            d.is_dir().then_some(d)
        }
    }
}

fn dropins(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "sh"))
        .collect();
    files.sort();
    Ok(files)
}

fn executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Run the post-create dropins in lexical order. A non-zero exit aborts the
/// rest; the worktree is left in place so the failure can be fixed by hand.
pub fn run_post_create(ctx: &HookCtx) -> Result<()> {
    let Some(dir) = hooks_dir() else {
        eprintln!("warning: no hooks directory found, skipping post-create hooks");
        return Ok(());
    };
    for script in dropins(&dir)? {
        let name = script.file_name().unwrap_or_default().to_string_lossy().into_owned();
        if !executable(&script) {
            eprintln!("warning: {name} is not executable, skipping");
            continue;
        }
        println!("==> hook {name}");
        let status = Command::new(&script)
            .current_dir(&ctx.dir)
            .stdin(Stdio::null())
            .env("WT_EVENT", ctx.event)
            .env("WT_REPO_ROOT", &ctx.repo_root)
            .env("WT_DIR", &ctx.dir)
            .env("WT_BRANCH", &ctx.branch)
            .env("WT_SLUG", &ctx.slug)
            .env("WT_BASE_REF", &ctx.base_ref)
            .env("WT_DEFAULT_BRANCH", &ctx.default_branch)
            .status()
            .with_context(|| format!("running hook {}", script.display()))?;
        if !status.success() {
            bail!(
                "hook {name} failed ({status}) — worktree {} left in place; fix and re-run the hook by hand",
                ctx.dir.display()
            );
        }
    }
    Ok(())
}
