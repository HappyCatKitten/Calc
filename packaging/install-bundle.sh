#!/usr/bin/env bash
set -euo pipefail
bundle="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
version="$(cat "$bundle/VERSION")"
data_root="${XDG_DATA_HOME:-$HOME/.local/share}"
install_dir="$data_root/calc/versions/$version"
mkdir -p "$install_dir" "$HOME/.local/bin" "$data_root/applications" "$data_root/icons/hicolor/scalable/apps"
if [[ "$bundle" != "$install_dir" ]]; then cp -a "$bundle/." "$install_dir/"; fi
ln -sfn "$install_dir/AppRun" "$HOME/.local/bin/calc"
cp "$install_dir/icon.svg" "$data_root/icons/hicolor/scalable/apps/io.github.HappyCatKitten.Calc.svg"
cat > "$data_root/applications/calc.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Calc
Comment=Matrix Multiplier
Exec="$install_dir/AppRun"
Icon=io.github.HappyCatKitten.Calc
Terminal=false
Categories=Utility;Calculator;
StartupWMClass=Calc
DESKTOP
if command -v update-desktop-database >/dev/null; then update-desktop-database "$data_root/applications"; fi
printf 'Installed Calc %s to %s\n' "$version" "$install_dir"
