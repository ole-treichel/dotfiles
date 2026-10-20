#!/usr/bin/env bash
# Installs the Firefox add-ons in this repo: the Oxide theme (firefox/oxide)
# and the new tab page (firefox/newtab). Fedora's Firefox build loads unsigned
# add-ons from the system extension scope (MOZ_UNSIGNED_SYSTEM_SCOPE), so no
# AMO signing and no web-ext. The theme carries both palettes (`theme` +
# `dark_theme`); Firefox picks one from the system color-scheme. See
# docs/oxide-theme.md and docs/firefox-newtab.md.
#
# One-time. Needs sudo for the symlinks. Firefox must be closed so prefs.js
# can be edited.
#
# macOS: Mozilla builds require signing in every scope except Developer
# Edition / Nightly, which honor xpinstall.signatures.required=false. There
# each add-on goes into the profile as an XPI (like thunderbird/install_theme.sh).
# Rerun after editing an add-on.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ADDON_DIRS=("$REPO_DIR/firefox/oxide" "$REPO_DIR/firefox/newtab")

# browser_specific_settings.gecko.id. The installed file or link must carry it
# as its name.
addon_id() {
  sed -n 's/^ *"id": *"\([^"]*\)".*/\1/p' "$1/manifest.json"
}

if [ "$(uname)" = "Darwin" ]; then
  PROFILES_DIR="$HOME/Library/Application Support/Firefox"
  if pgrep -q -f "Firefox.*\.app/Contents/MacOS/firefox"; then
    echo "Firefox is running. Close it and rerun." >&2
    exit 1
  fi
  awk -F= '/^\[Install/ { inst = 1; next } /^\[/ { inst = 0 } inst && $1 == "Default" { print $2 }' \
    "$PROFILES_DIR/profiles.ini" | while IFS= read -r p; do
    dir="$PROFILES_DIR/$p"
    [ -f "$dir/prefs.js" ] || continue
    mkdir -p "$dir/extensions"
    for addon in "${ADDON_DIRS[@]}"; do
      xpi="$dir/extensions/$(addon_id "$addon").xpi"
      rm -f "$xpi"
      (cd "$addon" && zip -q -X -r "$xpi" . -x '.DS_Store')
      echo "installed $xpi"
    done
    sed -i '' '/"xpinstall\.signatures\.required"/d' "$dir/prefs.js"
    echo 'user_pref("xpinstall.signatures.required", false);' >>"$dir/prefs.js"
    # Profile scope is scanned only when this startup cache is missing.
    rm -f "$dir/addonStartup.json.lz4"
  done
  echo "Start Firefox Developer Edition. Enable Oxide and New Tab Page in about:addons (sideloaded add-ons start disabled)."
  exit 0
fi
SYSTEM_SCOPE_DIR="/usr/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}"
PROFILES_DIR="$HOME/.mozilla/firefox"
# SCOPE_SYSTEM. Firefox scans this scope on every start, so edits to the
# add-ons apply after a restart instead of after the next Firefox update.
SCAN_PREF='user_pref("extensions.startupScanScopes", 8);'

# `firefox --dbus-service` (GNOME search provider) holds no profile; ignore it.
if pgrep -a -x firefox | grep -qv -- '--dbus-service'; then
  echo "Firefox is running. Close it and rerun." >&2
  exit 1
fi

for addon in "${ADDON_DIRS[@]}"; do
  link="$SYSTEM_SCOPE_DIR/$(addon_id "$addon")"
  sudo ln -sfn "$addon" "$link"
  echo "linked $link -> $addon"
done

# Profiles that are some installation's default (profiles.ini [Install*] Default=).
mapfile -t profiles < <(
  awk -F= '/^\[Install/ { inst = 1; next } /^\[/ { inst = 0 } inst && $1 == "Default" { print $2 }' \
    "$PROFILES_DIR/profiles.ini"
)
for p in "${profiles[@]}"; do
  prefs="$PROFILES_DIR/$p/prefs.js"
  [ -f "$prefs" ] || continue
  sed -i '/"extensions\.startupScanScopes"/d' "$prefs"
  echo "$SCAN_PREF" >>"$prefs"
  echo "set extensions.startupScanScopes=8 in $prefs"
done

echo "Start Firefox. If Oxide is not active: about:addons > Themes > Oxide > Enable."
echo "First new tab: Firefox asks to confirm the extension-controlled new tab page; keep it."
