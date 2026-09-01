#!/usr/bin/env bash
# Sign extension/ through AMO and drop the signed .xpi in dist/.
#
# Needed only for a restart-surviving install. Release Firefox hard-codes
# signature enforcement — `xpinstall.signatures.required` is ignored there — so
# an AMO signature is the only way to install this permanently short of running
# Developer Edition / Nightly / ESR. The channel is `unlisted` (self
# distribution): no public listing, no human review, automated signing.
#
# Credentials come from https://addons.mozilla.org/developers/addon/api/key/
# and are read from the environment, never stored in the repo:
#
#   export AMO_JWT_ISSUER=user:12345678:123
#   export AMO_JWT_SECRET=...
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

: "${AMO_JWT_ISSUER:?set AMO_JWT_ISSUER (JWT issuer from the AMO API key page)}"
: "${AMO_JWT_SECRET:?set AMO_JWT_SECRET (JWT secret from the AMO API key page)}"

version="$(jq -r .version "$repo/extension/manifest.json")"

# AMO refuses a version it has already signed, and the failure arrives several
# seconds into the upload. Catch the common case — forgetting the bump — before
# spending the round trip on it.
if compgen -G "$repo/dist/*-$version.xpi" >/dev/null; then
  echo "already signed $version (dist/$(basename "$(compgen -G "$repo/dist/*-$version.xpi" | head -1)"))" >&2
  echo "bump \"version\" in extension/manifest.json before re-signing." >&2
  exit 1
fi

mkdir -p "$repo/dist"

# Credentials go through WEB_EXT_* env vars rather than --api-key/--api-secret,
# which would put the secret in argv where any local process can read it off
# `ps`. web-ext accepts a WEB_EXT_-prefixed env var for every CLI option.
echo "==> signing $version (unlisted)"
WEB_EXT_API_KEY="$AMO_JWT_ISSUER" \
WEB_EXT_API_SECRET="$AMO_JWT_SECRET" \
  npx --yes web-ext sign \
    --source-dir "$repo/extension" \
    --artifacts-dir "$repo/dist" \
    --channel=unlisted

cat <<EOF

==> signed
Install permanently: about:addons -> gear -> Install Add-on From File ->
  $repo/dist/  (the *-$version.xpi just written)

Survives restarts. Remove the temporary add-on from about:debugging first.
EOF
