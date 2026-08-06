#!/usr/bin/env bash
# docs/<slug>/{prd.md,knowledge.md}. An empty branch identical to its base
# cannot have a PR, so these files are what make the first commit possible.
set -euo pipefail

[ "$WT_EVENT" = new ] || exit 0
# Gate on repo shape: no docs/ at the repo root, no scaffold.
[ -d docs ] || exit 0

dest="docs/$WT_SLUG"
mkdir -p "$dest"

case "$WT_SLUG" in
  feat-*)
    tpl="docs/templates/minimal-prd.md"
    if [ ! -f "$dest/prd.md" ]; then
      if [ -f "$tpl" ]; then
        cp "$tpl" "$dest/prd.md"
      else
        cat > "$dest/prd.md" <<EOF
# PRD: $WT_SLUG

> Kurzbeschreibung in 1–2 Sätzen.

**Status:** Entwurf

## Anforderungen

- [ ]

## Offene Fragen

- [ ]
EOF
      fi
    fi
    ;;
esac

if [ ! -f "$dest/knowledge.md" ]; then
  cat > "$dest/knowledge.md" <<EOF
# Knowledge: $WT_SLUG

Was beim Arbeiten an diesem Branch gelernt wurde — Fundstellen im Code,
Entscheidungen, Stolperfallen.

## Codestellen

## Entscheidungen

## Stolperfallen
EOF
fi

echo "scaffolded $dest"
