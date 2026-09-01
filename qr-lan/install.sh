#!/usr/bin/env bash
# Build the qr-lan companion and register it as a login service.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
binary="$repo/companion/target/release/qr-lan"

echo "==> building"
cargo build --release --manifest-path "$repo/companion/Cargo.toml"

echo "==> linking $HOME/.local/bin/qr-lan"
mkdir -p "$HOME/.local/bin"
ln -sfn "$binary" "$HOME/.local/bin/qr-lan"

case "$(uname -s)" in
  Linux)
    unit="$HOME/.config/systemd/user/qr-lan.service"
    echo "==> linking $unit"
    mkdir -p "$(dirname "$unit")"
    ln -sfn "$repo/service/qr-lan.service" "$unit"
    systemctl --user daemon-reload
    systemctl --user enable --now qr-lan.service
    systemctl --user --no-pager status qr-lan.service | head -3
    ;;
  Darwin)
    label="de.pixelwerte.qr-lan"
    plist="$HOME/Library/LaunchAgents/$label.plist"
    echo "==> linking $plist"
    mkdir -p "$(dirname "$plist")"
    ln -sfn "$repo/service/$label.plist" "$plist"
    launchctl bootout "gui/$UID/$label" 2>/dev/null || true
    launchctl bootstrap "gui/$UID" "$plist"
    ;;
  *)
    echo "unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac

echo "==> packaging extension"
xpi="$("$repo/package.sh")"

cat <<EOF

==> done
Companion listening on http://127.0.0.1:48213

Load the extension once per machine.

Chrome/Chromium — survives restarts:
  chrome://extensions -> Developer mode -> Load unpacked ->
  $repo/extension

Firefox — pick one:
  a) about:debugging#/runtime/this-firefox -> Load Temporary Add-on ->
     $xpi
     Pick the .xpi, NOT extension/manifest.json. Under a Flatpak Firefox the
     file picker goes through the XDG desktop portal, which exports only the
     one file you select — picking the manifest gives Firefox an extension
     root containing nothing else, and the popup renders empty with no error.
     Re-run package.sh after editing the extension. Gone on browser restart.
  b) web-ext run --source-dir $repo/extension
     Loads from the directory, bypassing the portal entirely, and reloads on
     save. Best for iterating. Needs the repo readable by the sandbox:
     flatpak override --user --filesystem=$repo:ro org.mozilla.firefox
  c) Permanent, on any channel: AMO-sign it, then install from about:addons.
       export AMO_JWT_ISSUER=... AMO_JWT_SECRET=...   # addons.mozilla.org
       $repo/sign.sh
     Survives restarts. Costs a version bump + re-sign per edit, so (a)/(b)
     stay the better loop while iterating.
  Release and Beta Firefox refuse unsigned add-ons permanently and ignore
  xpinstall.signatures.required, so (c) is the only permanent option there.
  Developer Edition, Nightly and ESR can instead set that pref to false in
  about:config and install the unsigned .xpi directly.

Keyboard shortcut: none by default — assign one under about:addons ->
Manage Extension Shortcuts (Firefox) or chrome://extensions/shortcuts.
EOF
