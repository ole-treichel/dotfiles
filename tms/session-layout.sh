#!/usr/bin/env bash
# tms-layout — give a freshly created tmux session the standard 3-window layout:
#
#   1 nvim   50/50 vertical split, `nvim .` on top, bare shell below
#   2 git    shell running lazygit
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

# Window 1: nvim on top, shell below. Renaming also stops tmux from
# auto-renaming the window after whatever process runs in it.
window=$(tmux list-windows -t "=$session" -F '#{window_id}' | head -1)
tmux rename-window -t "$window" nvim
editor=$(tmux list-panes -t "$window" -F '#{pane_id}' | head -1)
tmux split-window -v -c "$path" -t "$editor"
tmux send-keys -t "$editor" 'nvim .' Enter

for spec in "git:lazygit" "ai:claude"; do
  name=${spec%%:*}
  command=${spec#*:}
  window=$(tmux new-window -a -d -t "$window" -n "$name" -c "$path" -P -F '#{window_id}')
  tmux send-keys -t "$window" "$command" Enter
done

tmux select-window -t "=$session:^"
tmux select-pane -t "$editor"
