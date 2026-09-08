#!/usr/bin/env bash
# Commit the scaffold, push the branch, open the PR. Lives in a hook rather than
# in the binary so there is no `gh` dependency in Rust and the commit message or
# PR body can change without a rebuild.
set -euo pipefail

[ "$WT_EVENT" = new ] || exit 0

git add -A
if git diff --cached --quiet; then
  echo "nothing scaffolded, no commit"
else
  git commit -m "init"
fi

# Unconditional, even with nothing of our own to commit: without an upstream
# the first plain `git push` in the new worktree fails, and setting one is the
# whole point of -u. A repo with no docs/ at its root scaffolds nothing, and
# used to leave the branch here with no upstream and no PR.
git push -u origin "$WT_BRANCH"

# GitHub rejects a PR with "no commits between", so there is nothing to open
# until the branch is actually ahead of its base.
if [ "$(git rev-list --count "origin/$WT_DEFAULT_BRANCH..HEAD")" -eq 0 ]; then
  echo "no commits ahead of $WT_DEFAULT_BRANCH, no PR yet"
  exit 0
fi

command -v gh >/dev/null || exit 0

if [ -f "docs/$WT_SLUG/prd.md" ]; then
  body="See docs/$WT_SLUG/prd.md"
elif [ -d "docs/$WT_SLUG" ]; then
  body="See docs/$WT_SLUG/"
else
  body="$WT_SLUG"
fi

gh pr create \
  --base "$WT_DEFAULT_BRANCH" \
  --title "$WT_SLUG" \
  --body "$body"
