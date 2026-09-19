# Third-party notices

Project-authored code is MIT licensed. Bundled third-party components retain their own licenses.

## Qt 6.8.3

The binary release dynamically bundles unmodified Qt 6.8.3 libraries, QML modules and plugins. Applicable modules are used under LGPL-3.0; the license and its GPL-3.0 reference text are included under `licenses/`. Qt copyright belongs to The Qt Company Ltd and other contributors. Module SBOMs, copyright notices, and extracted third-party license texts are included in the binary package under `licenses/qt/`.

The exact Qt source release is available at:

- https://download.qt.io/archive/qt/6.8/6.8.3/single/qt-everywhere-src-6.8.3.tar.xz
- https://download.qt.io/archive/qt/6.8/6.8.3/submodules/

Qt is dynamically linked and can be replaced: substitute compatible rebuilt Qt shared libraries in `lib/` and corresponding modules/plugins in `qml/` and `plugins/`, or rebuild the open-source application against your Qt build. There are no signatures or locks preventing replacement. This application's license does not restrict reverse engineering for debugging modifications to the LGPL libraries.

## ICU 73.2

Qt's matching ICU runtime is included. Copyright and license: `licenses/ICU-73.2.txt`. Corresponding source: https://github.com/unicode-org/icu/releases/tag/release-73-2.

## Rust dependencies

Dependency versions are locked in `Cargo.lock`. Their license texts are bundled under `licenses/rust/`. They include serde, serde_json, serde_derive, proc-macro2, quote, syn, unicode-ident, memchr, itoa, zmij. They are licensed under MIT and/or Apache-2.0, with unicode-ident additionally using Unicode-3.0.

