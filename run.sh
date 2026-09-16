#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
QT_PREFIX="${QT_PREFIX:-$HOME/.local/share/Qt/6.8.3/gcc_64}"
export LD_LIBRARY_PATH="$QT_PREFIX/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export QT_QUICK_CONTROLS_STYLE=Basic
export QT_PLUGIN_PATH="$QT_PREFIX/plugins"
export QML_IMPORT_PATH="$QT_PREFIX/qml"
exec ./build/brainfuck-calculator "$@"
