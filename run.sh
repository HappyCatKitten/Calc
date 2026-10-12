#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
QT_PREFIX="${QT_PREFIX:-$HOME/.local/share/Qt/6.8.3/gcc_64}"
qt_lib="$QT_PREFIX/lib"
qt_plugins="$QT_PREFIX/plugins"
qt_qml="$QT_PREFIX/qml"
if [[ -d "$QT_PREFIX/lib/x86_64-linux-gnu/qt6" ]]; then
  qt_lib="$QT_PREFIX/lib/x86_64-linux-gnu"
  qt_plugins="$qt_lib/qt6/plugins"
  qt_qml="$qt_lib/qt6/qml"
fi
export LD_LIBRARY_PATH="$qt_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export QT_QUICK_CONTROLS_STYLE=Basic
export QT_PLUGIN_PATH="$qt_plugins"
export QML_IMPORT_PATH="$qt_qml"
exec ./build/calc "$@"
