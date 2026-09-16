# Brainfuck Calculator 1.0.0

A native Linux calculator with an actual Brainfuck numeric engine, a Qt/QML interface, and a Rust interpreter/parser wrapper.

- Standard and optional scientific keypads; comma digit grouping.
- Percentages, parentheses, memory, repeated equals, undo/redo.
- Saved searchable history and keyboard/clipboard controls.
- Readable options, scientific on/off, and always-on-top.

## Downloads

- **`brainfuck-calculator_1.0.0_amd64.deb`** — install with `sudo apt install ./brainfuck-calculator_1.0.0_amd64.deb`.
- **`brainfuck-calculator-1.0.0-linux-x86_64.tar.gz`** — extract and run `./AppRun`, or `./install.sh` for a per-user installation.
- **`SHA256SUMS`** — download integrity checks.

Both binary packages include the dynamically linked Qt 6.8.3 runtime and third-party notices. Tested on Pop!_OS 22.04/X11 with glibc 2.35. Wayland plugins are included but Wayland has not been validated.

## Validation and limits

50 Rust test functions; 48,234 counted campaign cases; 15,000 randomized formatting assertions; native-window calculation and panel checks. Counts overlap as explained in the repository's validation report.

The engine uses wide integer Brainfuck cells and 24-place fixed-point arithmetic with an f64 serialization boundary. Parsing and app state remain Rust. Extremely small operands, invalid domains, execution limits, and radian angles beyond 10^12 produce explicit errors. This is not an arbitrary-precision financial calculator.

Qt source and license information are linked in `THIRD_PARTY_NOTICES.md` and included with the binaries. The repository contains screenshots captured from the running release.
