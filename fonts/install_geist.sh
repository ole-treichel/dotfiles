#!/usr/bin/env bash
# Installs Geist (Fedora package vercel-geist-vf-fonts, variable font) and sets
# it as GNOME interface and document font (Tweaks > Fonts > Interface Text /
# Document Text). Monospace font is left unchanged. See docs/ui-font.md.
#
# Idempotent. Needs sudo only when the package is not yet installed.
set -euo pipefail

PACKAGE="vercel-geist-vf-fonts"
FAMILY="Geist"
# Pango font description, same format Tweaks writes for a variable font.
FONT="$FAMILY Medium 10 @wght=500"

if ! rpm -q "$PACKAGE" >/dev/null; then
  sudo dnf install -y "$PACKAGE"
  fc-cache -f
fi

if ! fc-list : family | grep -qx "$FAMILY"; then
  echo "fontconfig does not list family '$FAMILY'." >&2
  exit 1
fi

gsettings set org.gnome.desktop.interface font-name "$FONT"
gsettings set org.gnome.desktop.interface document-font-name "$FONT"

echo "font-name:          $(gsettings get org.gnome.desktop.interface font-name)"
echo "document-font-name: $(gsettings get org.gnome.desktop.interface document-font-name)"
