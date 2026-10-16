#!/usr/bin/env bash
# Install the Oxide theme into each Obsidian vault and select it.
#
# Copies (does not symlink) obsidian/oxide into <vault>/.obsidian/themes/Oxide:
# the vaults sync to GitHub/mobile via obsidian-git, where a symlink into this
# repo would not resolve. Re-run after editing obsidian/oxide/theme.css.
#
# Sets "cssTheme": "Oxide" in <vault>/.obsidian/appearance.json and leaves
# every other key alone ("theme": "system" makes Obsidian follow the GNOME
# color-scheme, so light/dark needs no theme-cli surface).

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
THEME_SRC="${REPO_DIR}/obsidian/oxide"
VAULTS=(
  "${HOME}/workspace/obsidian/work"
  "${HOME}/workspace/obsidian/private"
)

for vault in "${VAULTS[@]}"; do
  config_dir="${vault}/.obsidian"
  if [[ ! -d "${config_dir}" ]]; then
    echo "skip: ${vault} (no .obsidian directory)" >&2
    continue
  fi

  theme_dst="${config_dir}/themes/Oxide"
  mkdir -p "${theme_dst}"
  cp -f "${THEME_SRC}/manifest.json" "${THEME_SRC}/theme.css" "${theme_dst}/"

  appearance="${config_dir}/appearance.json"
  python3 -I - "${appearance}" <<'PY'
import json, os, sys

path = sys.argv[1]
raw = ""
if os.path.exists(path):
    with open(path, encoding="utf-8") as f:
        raw = f.read()
data = json.loads(raw) if raw.strip() else {}
data["cssTheme"] = "Oxide"
data.setdefault("theme", "system")
out = json.dumps(data, indent=2, ensure_ascii=False)
if raw.endswith("\n"):
    out += "\n"
with open(path, "w", encoding="utf-8") as f:
    f.write(out)
PY

  echo "installed: ${theme_dst} (cssTheme=Oxide)"
done
