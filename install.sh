#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)"
bundle="dist/calc-$version-linux-x86_64"
if [[ ! -x "$bundle/install.sh" ]]; then ./build.sh; python3 scripts/package.py; fi
exec "$bundle/install.sh"
