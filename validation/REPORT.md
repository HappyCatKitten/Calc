# Expanded Brainfuck calculator validation

**Result: passed after fixes.** The final release run passes all 50 Rust test functions and all formatting assertions. The rebuilt Qt application also passes direct native UI checks.

## Counted test campaigns

These counts represent individual cases inside parameterized tests, not separate Rust test functions. Counts are from the final run only, not accumulated reruns.

| Campaign | Cases |
| --- | ---: |
| Random signed decimal arithmetic | 12,000 |
| Random scientific functions, degrees and radians | 5,389 |
| Fractional/integer powers and factorials | 771 |
| Invalid domains and nonfinite operands | 603 |
| Difficult numerical boundaries | 27 |
| Generated valid expressions | 2,000 |
| Malformed/Unicode/null input without panics | 10,000 |
| Memory/history/action sequences | 300 |
| Literal versus optimized divmod | 8,192 |
| Literal versus optimized multiplication | 3,072 |
| Literal versus optimized affine transfers | 880 |
| **Counted Rust campaign cases** | **43,234** |
| Formatting/normalization scenarios | 5,000 |
| **Combined counted cases** | **48,234** |

Formatting scenarios make 15,000 assertions, in addition to the original eight format checks. Other existing unit tests and new focused regressions are not included in the campaign total. Numerical campaigns use deterministic seeds and explicit absolute/relative tolerances, not exact equality for transcendental floating-point references. Malformed-input fuzzing asserts absence of panics; it does not claim every random string should be rejected.

## Issues found and fixed

1. **Repeated equals confused unary negatives with subtraction.** `5 × −2 = =` now gives `20`; `5 ÷ −2 = =` gives `1.25`.
2. **Current-operand operations could lose the meaning of a leading negative.** Applying square root to `−4` now reports a domain error rather than evaluating `−√4`.
3. **Interpreter optimizations could erase errors.** Opposing instruction runs no longer cancel away invalid intermediate pointer moves or decrements. Affine loops retain their full pointer span, avoid unsafe mixed-sign folding, and skip all effects when their control cell is zero. The reference path now executes each instruction literally.
4. **Cell-size guards missed two mutation paths.** Increment and multiplication accumulation now enforce the same 4,096-bit limit as other operations.
5. **Very large radian angles could return an inaccurate phase.** Brainfuck now rejects magnitudes above 10^12 radians with an explicit accuracy-limit message. Degree mode retains large-angle reduction and was separately checked up to magnitudes of 10^100.

## Native Qt checks

Tested the rebuilt release executable on an isolated Xvfb X11 display, using real keyboard input and copying the visible result via the system clipboard:

- `5 * -2`, then equals again → `20`.
- `5 / -2`, then equals again → `1.25`.
- `sqrt(-4)` → domain error.
- `1 / 0` → divide-by-zero error.
- `sin(30) + sqrt(81)` → `9.5`.
- `1,234,567.89 * 2` → `2469135.78` (raw copied result).
- `200 + 10%` → `220`.
- History closes/reopens at 430/660 pixels; scientific mode opens/closes at 744/584 pixels.

QA used isolated data/config directories. User history was not cleared or populated by these tests.

## Reproduction and evidence

From the app directory:

```sh
cargo test --release --locked -- --nocapture
node tests/format.test.mjs
./build.sh
```

- [Full Rust test output](rust-tests.log)
- [Formatting output](format-tests.log)
- [Native UI calculation evidence](native-ui.json)

This is strong regression coverage, not an exhaustive proof of the calculator or interpreter. Arithmetic still uses the documented fixed-point Brainfuck engine and f64 serialization boundary. Wayland behavior and cross-machine packaging were not tested in this campaign.
