# Calc validation

The native Rust engine is covered by arithmetic, scientific-function, domain, parser, state-management and deferred-evaluation tests.

- Deterministic arithmetic and scientific comparisons, powers and factorials, large-degree angle reduction, invalid domains and boundary values.
- Generated valid expressions and malformed input without panics.
- Memory, undo/redo, bounded history, relative percentages, repeated equals and unary-negative regressions.
- Explicit checks that typing, editing, scientific keys, memory recall and history reuse do not evaluate until equals; errors also wait for equals.
- 5,000 formatting scenarios with 15,000 assertions, plus eight fixed checks.
- Native Qt/X11 checks for keyboard input, P/M/D/T shortcuts, scientific expression editing, history persistence, and visible results before and after Enter.

Commands: `cargo test --release --locked`, `node tests/format.test.mjs`, `python3 scripts/screenshots.py`.

Bundled-app smoke tests additionally check that the app uses its bundled Qt runtime instead of the developer SDK. Wayland remains unverified.
