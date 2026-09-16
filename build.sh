#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
node brainfuck/generate.mjs
QT_PREFIX="${QT_PREFIX:-$HOME/.local/share/Qt/6.8.3/gcc_64}"
if [[ ! -f Cargo.lock ]]; then cargo generate-lockfile; fi
cmake -S . -B build -G Ninja -DCMAKE_PREFIX_PATH="$QT_PREFIX" -DCMAKE_BUILD_TYPE=Release
cmake --build build
