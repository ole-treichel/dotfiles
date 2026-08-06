#!/usr/bin/env bash
# Copy the gitignored .env* files from the default-branch worktree into the new
# one — a fresh `git worktree add` produces a checkout that cannot run without
# them. Runs for both `wt new` and `wt get`.
set -euo pipefail

src="$WT_REPO_ROOT/$WT_DEFAULT_BRANCH"
[ -d "$src" ] || exit 0
[ "$src" != "$WT_DIR" ] || exit 0

copied=0
while IFS= read -r -d '' rel; do
  case "${rel##*/}" in .env*) ;; *) continue ;; esac
  case "$rel" in node_modules/*|*/node_modules/*) continue ;; esac
  [ ! -f "$WT_DIR/$rel" ] || continue
  mkdir -p "$WT_DIR/$(dirname "$rel")"
  cp "$src/$rel" "$WT_DIR/$rel"
  copied=$((copied + 1))
done < <(git -C "$src" ls-files -z --others --ignored --exclude-standard -- '*.env*')

echo "seeded $copied env file(s) from $WT_DEFAULT_BRANCH/"
