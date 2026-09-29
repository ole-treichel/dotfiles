# Bash config: public/private split

## Shape
- `bash/bashrc` (repo, public) → symlinked to `~/.bashrc`.
- `~/.bashrc.private` (not in repo, mode 600). Sourced as last line of `bash/bashrc`.

## Decisions
- **Single private file, not `~/.bashrc.d/`.** One explicit path; easier to find than a glob dir. `~/.bashrc.d/` loop kept (Fedora default) but unused.
- **Private sourced last.** Private can override public. Located after the interactive guard + `exec tmux`, so secrets load only in interactive shells — same as before the split.
- **Private = secrets + host details:** API keys (`ANTHROPIC_API_KEY`, `MOCO_API_KEY`), SSH aliases with IPs/hostnames/ports, paths into `workspace/private`.
- **Public = tool PATH setup, generic aliases/functions.** `DOCKER_GATEWAY_HOST=172.17.0.1` stays public (Docker default, not private).
- **Removed `h2` alias** (Shopify Hydrogen); unused.
- Old file kept as `~/.bashrc.bak`.

## Rules
- New secret or host-specific entry → `~/.bashrc.private`, never `bash/bashrc`.
- Before commit: `grep -niE 'key|token|secret|pass|[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+' bash/bashrc`.
- Installers appending to `~/.bashrc` write into the repo file via the symlink → review diff.

## Non-goals
- No secret manager / encrypted file in repo.
- SSH aliases not migrated to `~/.ssh/config` (possible later).
- `~/.bash_profile` not moved.
