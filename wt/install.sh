#!/usr/bin/env bash
# Build wt and link it into ~/.local/bin.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
binary="$repo/target/release/wt"

echo "==> building"
cargo build --release --manifest-path "$repo/Cargo.toml"

echo "==> making hooks executable"
chmod +x "$repo"/hooks/post-create.d/*.sh

echo "==> linking $HOME/.local/bin/wt"
mkdir -p "$HOME/.local/bin"
ln -sfn "$binary" "$HOME/.local/bin/wt"

cat <<EOF

==> done
Hooks run from $repo/hooks/post-create.d
(resolved from the binary's real path, so the symlink is fine)
EOF
