#!/usr/bin/env bash
# Commit the scaffold, push the branch, open the PR. Lives in a hook rather than
# in the binary so there is no `gh` dependency in Rust and the commit message or
# PR body can change without a rebuild.
set -euo pipefail

[ "$WT_EVENT" = new ] || exit 0

git add -A
if git diff --cached --quiet; then
  echo "nothing scaffolded, no commit"
  exit 0
fi
git commit -m "init"
git push -u origin "$WT_BRANCH"

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
