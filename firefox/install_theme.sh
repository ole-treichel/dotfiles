#!/usr/bin/env bash
# Installs the Oxide Firefox theme (firefox/oxide) into Firefox's system
# extension scope. Fedora's Firefox build loads unsigned add-ons from that
# scope (MOZ_UNSIGNED_SYSTEM_SCOPE), so no AMO signing and no web-ext.
# The theme carries both palettes (`theme` + `dark_theme`); Firefox picks one
# from the system color-scheme. See docs/oxide-theme.md.
#
# One-time. Needs sudo for the symlink. Firefox must be closed so prefs.js
# can be edited.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
THEME_DIR="$REPO_DIR/firefox/oxide"
ADDON_ID="oxide@theme-cli"
SYSTEM_SCOPE_DIR="/usr/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}"
PROFILES_DIR="$HOME/.mozilla/firefox"
# SCOPE_SYSTEM. Firefox scans this scope on every start, so edits to
# firefox/oxide apply after a restart instead of after the next Firefox update.
SCAN_PREF='user_pref("extensions.startupScanScopes", 8);'

# `firefox --dbus-service` (GNOME search provider) holds no profile; ignore it.
if pgrep -a -x firefox | grep -qv -- '--dbus-service'; then
  echo "Firefox is running. Close it and rerun." >&2
  exit 1
fi

sudo ln -sfn "$THEME_DIR" "$SYSTEM_SCOPE_DIR/$ADDON_ID"
echo "linked $SYSTEM_SCOPE_DIR/$ADDON_ID -> $THEME_DIR"

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
