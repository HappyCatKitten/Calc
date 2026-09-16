#!/usr/bin/env bash
set -euo pipefail
bundle="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
version="$(cat "$bundle/VERSION")"
data_root="${XDG_DATA_HOME:-$HOME/.local/share}"
install_dir="$data_root/brainfuck-calculator/versions/$version"
mkdir -p "$install_dir" "$HOME/.local/bin" "$data_root/applications" "$data_root/icons/hicolor/scalable/apps"
if [[ "$bundle" != "$install_dir" ]]; then cp -a "$bundle/." "$install_dir/"; fi
ln -sfn "$install_dir/AppRun" "$HOME/.local/bin/brainfuck-calculator"
cp "$install_dir/icon.svg" "$data_root/icons/hicolor/scalable/apps/brainfuck-calculator.svg"
cat > "$data_root/applications/brainfuck-calculator.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Brainfuck Calculator
Comment=A native calculator with a Brainfuck numeric engine
Exec="$install_dir/AppRun"
Icon=brainfuck-calculator
Terminal=false
Categories=Utility;Calculator;
StartupWMClass=Brainfuck Calculator
DESKTOP
# Hide the superseded development launcher without deleting user data.
if [[ -f "$data_root/applications/obsidian-calculator.desktop" ]]; then
  if ! grep -q '^Hidden=true$' "$data_root/applications/obsidian-calculator.desktop"; then
    printf '\nHidden=true\n' >> "$data_root/applications/obsidian-calculator.desktop"
  fi
fi
if command -v update-desktop-database >/dev/null; then update-desktop-database "$data_root/applications"; fi
printf 'Installed Brainfuck Calculator %s to %s\n' "$version" "$install_dir"
