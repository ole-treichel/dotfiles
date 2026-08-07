#!/usr/bin/env bash
# Link the tms config and the session layout script into place.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> linking $HOME/.config/tms/config.toml"
mkdir -p "$HOME/.config/tms"
ln -sfn "$repo/config.toml" "$HOME/.config/tms/config.toml"

echo "==> linking $HOME/.local/bin/tms-layout"
chmod +x "$repo/session-layout.sh"
mkdir -p "$HOME/.local/bin"
ln -sfn "$repo/session-layout.sh" "$HOME/.local/bin/tms-layout"

cat <<EOF

==> done
The layout runs from tmux's session-created hook (.tmux.conf).
Reload it in a running server with: tmux source-file ~/.tmux.conf
EOF
