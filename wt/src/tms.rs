use std::process::{Command, Stdio};

/// `tms refresh` — adds windows for new worktrees in the current session.
/// Best effort: no tmux, no tms, or a failing refresh must not fail the command.
pub fn refresh() {
    if std::env::var_os("TMUX").is_none() {
        return;
    }
    let status = Command::new("tms")
        .arg("refresh")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(_) => eprintln!("warning: tms refresh failed"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => eprintln!("warning: tms refresh: {e}"),
    }
}
