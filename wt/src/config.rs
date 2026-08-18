use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub struct Config {
    pub vault: PathBuf,
}

/// Reads `~/.config/wt/config.toml` — currently just the Obsidian vault path.
/// Arbitrary per-machine state with no sensible default, so unlike
/// `WT_HOOKS_DIR` this has no env-var fallback; see docs/wt-vault.md.
pub fn load() -> Result<Config> {
    let home = std::env::var_os("HOME").context("$HOME is not set")?;
    let path = PathBuf::from(home).join(".config/wt/config.toml");
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading {} — run wt/install.sh", path.display()))?;
    let vault = vault_value(&text)
        .with_context(|| format!("no `vault = \"...\"` line in {}", path.display()))?;
    Ok(Config { vault: expand_tilde(&vault) })
}

fn vault_value(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let line = line.split('#').next().unwrap_or("").trim();
        let rest = line.strip_prefix("vault")?.trim_start();
        let rest = rest.strip_prefix('=')?.trim();
        let rest = rest.strip_prefix('"')?;
        rest.strip_suffix('"').map(str::to_string)
    })
}

fn expand_tilde(path: &str) -> PathBuf {
    match std::env::var_os("HOME").zip(path.strip_prefix("~/")) {
        Some((home, rest)) => PathBuf::from(home).join(rest),
        None => PathBuf::from(path),
    }
}

#[cfg(test)]
mod tests {
    use super::vault_value;

    #[test]
    fn reads_the_vault_line() {
        assert_eq!(
            vault_value("vault = \"~/workspace/obsidian/work\"\n"),
            Some("~/workspace/obsidian/work".to_string())
        );
        assert_eq!(vault_value("# vault = \"nope\"\n"), None);
        assert_eq!(vault_value("other = \"thing\"\n"), None);
    }
}
