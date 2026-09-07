#!/usr/bin/env bash
# tms-layout — give a freshly created tmux session the standard 3-window layout:
#
#   1 nvim   vertical split, `nvim .` on top (~75%), bare shell below (~25%)
#   2 git    lazygit, single pane
#   3 ai     shell running claude
#
# Everything runs inside a normal shell, so quitting an app leaves a prompt.
# Wired to tmux's session-created hook in .tmux.conf; also runnable by hand:
#
#   tms-layout [session]
#
set -euo pipefail

session="${1:-}"
if [ -z "$session" ]; then
  session=$(tmux display-message -p '#{session_name}')
fi

tmux has-session -t "=$session" 2>/dev/null || {
  echo "tms-layout: no such session: $session" >&2
  exit 1
}

config="${TMS_CONFIG_FILE:-$HOME/.config/tms/config.toml}"

# The default session is a landing spot, not a project — leave it alone.
if [ -r "$config" ]; then
  default_session=$(sed -n 's/^[[:space:]]*default_session[[:space:]]*=[[:space:]]*"\(.*\)"[[:space:]]*$/\1/p' "$config")
  [ -n "$default_session" ] && [ "$session" = "$default_session" ] && exit 0
fi

# Already arranged (or a hand-built session) — never stomp existing windows/panes.
# This cannot be the worktree test: the hook fires before tms adds its worktree
# windows, so at that moment even a worktree root still has a single window.
windows=$(tmux list-windows -t "=$session" -F '#{window_id}' | wc -l)
panes=$(tmux list-panes -t "=$session" -F '#{pane_id}' | wc -l)
if [ "$windows" -gt 1 ] || [ "$panes" -gt 1 ]; then
  exit 0
fi

path=$(tmux display-message -p -t "=$session" '#{session_path}')

# Roots that own several worktrees are tms's business: it opens one window per
# worktree there. Skip those; lay out everything else (plain repo, a single
# linked worktree, or a plain directory).
is_worktree_root() {
  local p=$1 gitdir common count
  git -C "$p" rev-parse --git-dir >/dev/null 2>&1 || return 1

  [ "$(git -C "$p" rev-parse --is-bare-repository)" = "true" ] && return 0

  gitdir=$(git -C "$p" rev-parse --absolute-git-dir)
  common=$(cd "$p" && cd "$(git rev-parse --git-common-dir)" && pwd)
  # A linked worktree has its own gitdir under the common one — always laid out.
  [ "$gitdir" != "$common" ] && return 1

  count=$(git -C "$p" worktree list --porcelain | grep -c '^worktree ' || true)
  [ "$count" -gt 1 ]
}

if is_worktree_root "$path"; then
  exit 0
fi

# Window 1: nvim on top (~75%), shell below (~25%). Renaming also stops tmux
# from auto-renaming the window after whatever process runs in it.
window=$(tmux list-windows -t "=$session" -F '#{window_id}' | head -1)
tmux rename-window -t "$window" nvim
editor=$(tmux list-panes -t "$window" -F '#{pane_id}' | head -1)
tmux split-window -v -l 25% -c "$path" -t "$editor"
tmux send-keys -t "$editor" 'nvim .' Enter

# Window 2: lazygit, single pane.
window=$(tmux new-window -a -d -t "$window" -n git -c "$path" -P -F '#{window_id}')
lazygit_pane=$(tmux list-panes -t "$window" -F '#{pane_id}' | head -1)
tmux send-keys -t "$lazygit_pane" lazygit Enter

# Claude asks "do you trust the files in this folder?" the first time it runs in
# a directory. Trust lives per-directory in ~/.claude.json, and its ancestor walk
# stops at the enclosing git root — so trusting ~/workspace once does not cover
# the repos inside it, and every new project asks again. tms only ever opens
# directories I picked myself, so answer it up front.
#
# Best effort: skipped without jq or a writable config, and a Claude session
# running elsewhere can rewrite the file and drop the key. Worst case the dialog
# shows up once.
pretrust_claude() {
  local dir=$1 real cfg tmp
  command -v jq >/dev/null 2>&1 || return 0
  cfg="${CLAUDE_CONFIG_DIR:-$HOME}/.claude.json"
  [ -f "$cfg" ] && [ -w "$cfg" ] || return 0
  real=$(cd "$dir" 2>/dev/null && pwd -P) || return 0
  tmp=$(mktemp "$cfg.XXXXXX") || return 0
  if jq --arg p "$real" '.projects[$p].hasTrustDialogAccepted = true' "$cfg" >"$tmp" &&
    [ -s "$tmp" ]; then
    chmod 600 "$tmp"
    mv "$tmp" "$cfg"
  else
    rm -f "$tmp"
  fi
}

# Window 3: shell running claude.
pretrust_claude "$path"
window=$(tmux new-window -a -d -t "$window" -n ai -c "$path" -P -F '#{window_id}')
tmux send-keys -t "$window" claude Enter

tmux select-window -t "=$session:^"
tmux select-pane -t "$editor"
