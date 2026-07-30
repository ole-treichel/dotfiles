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

cat <<EOF

==> done
Companion listening on http://127.0.0.1:48213

Load the extension once per machine:
  chrome://extensions -> Developer mode -> Load unpacked ->
  $repo/extension
EOF
