#!/usr/bin/env bash
# Installs the Oxide theme into Thunderbird (flatpak). Same manifest as
# Firefox (firefox/oxide): `theme` + `dark_theme`, so Thunderbird follows the
# system color-scheme by itself. Thunderbird does not require add-on
# signatures (xpinstall.signatures.required defaults to false), so the XPI is
# placed straight into the profile. See docs/oxide-theme.md.
#
# Rerun after editing firefox/oxide/manifest.json. Thunderbird must be closed.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
THEME_DIR="$REPO_DIR/firefox/oxide"
APP_ID="net.thunderbird.Thunderbird"
PROFILES_DIR="$HOME/.var/app/$APP_ID/.thunderbird"
ADDON_ID="oxide@theme-cli"

if flatpak ps --columns=application | grep -qx "$APP_ID"; then
  echo "Thunderbird is running. Close it and rerun." >&2
  exit 1
fi

# Profiles that are some installation's default (profiles.ini [Install*] Default=).
mapfile -t profiles < <(
  awk -F= '/^\[Install/ { inst = 1; next } /^\[/ { inst = 0 } inst && $1 == "Default" { print $2 }' \
    "$PROFILES_DIR/profiles.ini"
)
for p in "${profiles[@]}"; do
  dir="$PROFILES_DIR/$p"
  [ -d "$dir" ] || continue
  mkdir -p "$dir/extensions"
  xpi="$dir/extensions/$ADDON_ID.xpi"
  rm -f "$xpi"
  (cd "$THEME_DIR" && zip -q -X "$xpi" manifest.json)
  # Thunderbird scans the profile's extensions dir only when this startup
  # cache is missing or the build changes. It is rebuilt on the next start.
  rm -f "$dir/addonStartup.json.lz4"
  echo "installed $xpi"
done

echo "Start Thunderbird. If Oxide is not active: Add-ons and Themes > Themes > Oxide > Enable."
