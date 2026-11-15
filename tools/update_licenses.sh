#!/usr/bin/env sh
# Regenerate THIRD_PARTY_LICENSES.md, matching the CI check.
# usage: tools/update_licenses.sh
set -eu

VERSION="0.9.3" # keep in sync with .github/workflows/main.yaml
cd "$(dirname "$0")/.."

if ! cargo about --version 2>/dev/null | grep -qx "cargo-about $VERSION"; then
    echo "installing cargo-about $VERSION"
    cargo install --locked --features cli "cargo-about@$VERSION"
fi

cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md
echo "wrote THIRD_PARTY_LICENSES.md"
