#!/usr/bin/env bash
set -euo pipefail
bundle="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
version="$(cat "$bundle/VERSION")"
data_root="${XDG_DATA_HOME:-$HOME/.local/share}"
install_dir="$data_root/calc/versions/$version"
mkdir -p "$(dirname "$install_dir")" "$HOME/.local/bin" "$data_root/applications" "$data_root/icons/hicolor/scalable/apps"
if [[ "$bundle" != "$install_dir" ]]; then
  # Never overwrite a running executable or its mapped libraries in place.
  staging="$(mktemp -d "$(dirname "$install_dir")/.${version}.install.XXXXXX")"
  previous=""
  cleanup_install() {
    if [[ -n "$previous" && ! -e "$install_dir" ]]; then mv -- "$previous" "$install_dir"; fi
    if [[ -n "$staging" && -d "$staging" ]]; then rm -rf -- "$staging"; fi
  }
  trap cleanup_install EXIT
  cp -a "$bundle/." "$staging/"
  if [[ -e "$install_dir" ]]; then
    previous="$(mktemp -d "${install_dir}.previous.XXXXXX")"
    rmdir -- "$previous"
    mv -- "$install_dir" "$previous"
  fi
  mv -- "$staging" "$install_dir"
  staging=""
  trap - EXIT
  # Retain the old tree for running processes and rollback; leave user data alone.
fi
ln -sfn "$install_dir/AppRun" "$HOME/.local/bin/calc"
cp "$install_dir/icon.svg" "$data_root/icons/hicolor/scalable/apps/io.github.HappyCatKitten.Calc.svg"
cat > "$data_root/applications/calc.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Calc
Comment=Scientific calculator
Exec="$install_dir/AppRun"
Icon=io.github.HappyCatKitten.Calc
Terminal=false
Categories=Utility;Calculator;
StartupWMClass=Calc
DESKTOP
if command -v update-desktop-database >/dev/null; then update-desktop-database "$data_root/applications"; fi
printf 'Installed Calc %s to %s\n' "$version" "$install_dir"
