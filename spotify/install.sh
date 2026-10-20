#!/usr/bin/env bash
# Runs the Spotify flatpak under XWayland so GNOME draws the window title bar.
# On native Wayland Mutter provides no server-side decorations, and Spotify's
# CEF falls back to Chromium's blue frame. See docs/spotify-decorations.md.
#
# The manifest declares `fallback-x11`, which flatpak does not grant on a
# Wayland session, so X11 is granted explicitly. Restart Spotify to apply.
# Revert: flatpak override --user --reset com.spotify.Client
set -euo pipefail

APP_ID="com.spotify.Client"

if ! flatpak info "$APP_ID" >/dev/null 2>&1; then
  echo "$APP_ID is not installed." >&2
  exit 1
fi

flatpak override --user --nosocket=wayland --socket=x11 "$APP_ID"
flatpak override --user --show "$APP_ID"

echo "Restart Spotify to apply: flatpak kill $APP_ID"
