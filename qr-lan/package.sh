#!/usr/bin/env bash
# Zip extension/ into dist/qr-lan.xpi.
#
# Needed because Firefox here is a Flatpak: `about:debugging` -> Load Temporary
# Add-on opens the XDG desktop portal file picker, and the portal exports only
# the *single file* you select, at a synthetic /run/user/$UID/doc/<id>/ path.
# Pick extension/manifest.json and Firefox's extension root becomes a directory
# holding nothing but manifest.json — popup.html, popup.js and icons/ are all
# absent, so the popup renders as a genuinely empty document with no errors.
# An .xpi is one file, so the portal exports it whole.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="$repo/dist/qr-lan.xpi"

mkdir -p "$repo/dist"
rm -f "$out"
# -FS so a rebuild drops files deleted since the last one. manifest.json must
# sit at the archive root, hence the cd.
(cd "$repo/extension" && zip -q -r -FS "$out" . -x '.*' '*/.*')

echo "$out"
