# tms session layout — 3 windows for every new session

Picking a repo in `tms` used to drop you in a single blank shell. Now every new
tmux session opens as:

| # | name | contents |
| --- | --- | --- |
| 1 | `nvim` | vertical split — `nvim .` on top (~75%), bare shell below (~25%) |
| 2 | `git` | `lazygit`, single pane |
| 3 | `ai` | shell running `claude` |

Status: implemented. `./tms/install.sh` links `~/.local/bin/tms-layout` and
`~/.config/tms/config.toml`. The hook lives in `.tmux.conf`.

## Why a tmux hook and not tms

tms already has a create-script hook — on session creation it `send-keys`-runs
`<repo>/.tms-create`, or `session_configs.<session_name>.create_script` from its
config (`src/tmux.rs:137`, `src/session.rs:59`). Both are **per repo**: there is
no global default. Getting one layout for every repo through them would mean a
`.tms-create` file in every repo, or a config entry per repo.

The alternatives considered:

| Option | Verdict |
| --- | --- |
| tmux `session-created` hook → script | **Chosen.** No fork, survives `cargo install` upgrades, lives entirely in this repo, and covers every entry point (picker, marks, `clone-repo`) plus hand-made sessions |
| Patch tms with a `default_create_script` key | Cleanest semantics and upstreamable, but it means owning a build of a tool that's currently a plain `cargo install` |
| Wrap the keybind: `tms && tms-layout` | Only covers `prefix+p`, misses marks and `clone-repo` |
| `.tms-create` per repo | A file in every repo, forever |

Non-goal: no per-repo customisation. One layout for everything; a repo that
wants something else can still drop in its own `.tms-create`, which tms runs
independently of this.

## The worktree rule

With `list_worktrees = true`, tms itself opens **one window per worktree** for a
repo root that has them. Those sessions must not get the 3-window layout — but a
single worktree opened as its own session should.

So `tms-layout` skips a session when its `#{session_path}` is:

- a **bare repo** (`is-bare-repository` — the `wt` layout's `.bare` roots), or
- a **main worktree that has linked worktrees** (`git-dir == git-common-dir` and
  `git worktree list` has more than one entry).

Everything else is laid out: a plain repo, a **linked worktree** (its `git-dir`
differs from `git-common-dir`), or a plain directory such as a bookmark.

The obvious test — "does the session already have more than one window?" —
cannot work here. tms creates the session, tmux fires `session-created`, and
*only then* does tms add its worktree windows, so at hook time a worktree root
still looks like a one-window session. The check has to be a git question about
the path. The window count is still checked, but for a different reason: it
makes a manual `tms-layout` re-run a no-op instead of stomping a session you
have already arranged.

There is no race for the sessions that *are* laid out: tms's `set_up_tmux_env`
returns early for linked worktrees and for non-bare repos without worktrees, so
it never touches a session while the script is building it.

## Claude's trust dialog

Window 3 starts `claude` in a directory it has usually never seen, so every new
project opened the session with *"Do you trust the files in this folder?"*.

There is **no flag that skips only that dialog**. Checked against Claude Code
2.1.263, three things suppress it, and all three cost more than they are worth
here:

| Escape hatch | Why not |
| --- | --- |
| `-p` / non-TTY stdout | Documented in `--help`, but that is print mode, not an interactive session |
| `--dangerously-skip-permissions` | Disables *every* permission check, not just trust — the auto-mode classifier goes with it |
| `CLAUDE_CODE_SANDBOXED=1` | Internal escape hatch that changes other assumptions about the environment |

Trust is per directory, stored in `~/.claude.json` as
`projects["<realpath>"].hasTrustDialogAccepted`. It *is* inherited from ancestor
directories, but the walk up stops at the enclosing git root — so trusting
`~/workspace` once covers plain directories under it and **not** the repos
inside it, which is every project that matters. There is no once-and-for-all
answer to give.

So `tms-layout` writes the key itself, for the session path only, just before it
types `claude`. The permission model is untouched: only the "is this directory
mine?" question is pre-answered, and only for directories that reached the
script because they were picked in tms.

`pretrust_claude` is deliberately best-effort — it returns 0 without `jq`,
without a writable `~/.claude.json`, or if the path does not resolve, and it
writes through `mktemp` + `mv` at mode 600 so a failed `jq` can never truncate
the config. A Claude session running elsewhere holds the same file and can
rewrite it from its own in-memory copy, dropping the key; the cost of losing
that race is the dialog appearing once. Not worth a lock file.

The path is written as `pwd -P` output, because Claude keys the config on the
resolved realpath of its cwd.

## Decisions

| Area | Decision |
| --- | --- |
| Trigger | `set-hook -g session-created` with `run-shell -b`, so it never blocks the tmux server |
| Scope | Every new session, tms-made or hand-made. tms only ever creates sessions under `~/workspace` anyway, and a manual session in a repo wants the same layout |
| Default session | Skipped, by name, read from tms's `default_session` (`workspace`) rather than hardcoded — it is a landing spot, not a project |
| Apps in shells | `nvim`/`lazygit`/`claude` are typed into normal shells, not run as the window command. Quitting any of them leaves a usable prompt with the command in history, instead of destroying the window |
| Window names | Set with `-n` / `rename-window`, which also switches off tmux's automatic renaming, so window 2 stays `git` and does not become `lazygit` |
| Focus | Window 1, top pane |
| Idempotency | Refuses when the target session already has more than one window or pane |
| Claude trust | Pre-answered per session path in `~/.claude.json`, not `--dangerously-skip-permissions`. Only the trust question is skipped; permission prompts stay |
| Language | bash. It is a dozen tmux calls; `wt`'s Rust treatment would be ceremony |
| Install | `tms/install.sh` symlinks `~/.local/bin/tms-layout` and `~/.config/tms/config.toml`, same pattern as `wt` and `qr-lan` minus the build step |

## Files

- `tms/session-layout.sh` — the script (`tms-layout [session]`)
- `tms/install.sh` — symlinks
- `.tmux.conf` — the `session-created` hook, next to the tms keybinds
