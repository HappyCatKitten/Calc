# Brainfuck numeric engine

The calculator actually executes these `.bf` files. This is not a Brainfuck banner around the old Rust maths. The expression parser in `src/lib.rs` dispatches numeric operations to `brainfuck::calculate`; its arithmetic and scientific routines no longer call native floating-point arithmetic or libm. The VM has no calculator-specific opcode, function-call escape, or native-math fallback.

## What lives where

- `*.bf`: executable numeric routines, containing only Brainfuck instructions and a trailing newline.
- `generate.mjs`: build-time JavaScript macro assembler. It expands signed fixed-point algorithms into Brainfuck. It is never called to evaluate user calculations.
- `manifest.json`: reproducible tape layout, cell requirements and instruction counts.
- `../src/brainfuck.rs`: generic Brainfuck VM, loop optimizations, safety limits, and tape serialization.
- `../src/lib.rs`: Rust expression grammar, precedence, app state, undo/redo, history and display formatting. Parsing stays in Rust; numeric computation is Brainfuck.

## Dialect and calling convention

The language has only `><+-[],.`. Cells are arbitrary-width **nonnegative integers**, not wrapping eight-bit bytes. Decrement below zero and out-of-bounds tape movement are errors. The VM limits a run to 512 cells, 2 million optimized instructions, and 4,096-bit intermediate values. These are interpreter resource limits, not extra language instructions.

The app uses a tape ABI rather than printing/parsing a terminal protocol. On entry, all other cells are zero:

| Cell | Meaning |
| --- | --- |
| 0, 1 | A magnitude and negative flag |
| 2, 3 | B magnitude and negative flag |
| 4 | Degrees flag (1) or radians (0) |
| 5 | Error code output |
| 6, 7 | Result magnitude and negative flag |

Magnitudes are decimal fixed point with scale 10^24. For example, 1.5 enters as magnitude 1500000000000000000000000. The host only serializes operands, selects the program, runs the VM, and deserializes its result. No Brainfuck routines call host arithmetic functions. The VM itself necessarily uses native integer operations to implement Brainfuck instructions and optimize equivalent loops.

The existing f64 interface is retained, so this is not an arbitrary-precision user-facing calculator. Fixed-point intermediate division truncates. The display still rounds to about 13 significant digits. Inputs below 1e-24 and underflow in public multiplication, division, and power operations are explicit errors. Internal series terms truncate when they fall below fixed-point precision. Radian trig inputs above magnitude 10^12 are rejected because the fixed-point pi constant cannot provide reliable phase reduction there. Degree-mode reduction supports much larger inputs. These numerical semantics differ from the previous libm-backed engine.

## Algorithms

- Signed addition/subtraction use sign/magnitude comparisons and transfer loops.
- Multiplication uses repeated addition; division uses a Brainfuck divmod loop.
- Roots use integer Newton iteration on a scaled square.
- Integral powers use repeated squaring; fractional powers use exp(exponent × ln(base)).
- Logarithms normalize into [1, 2) and use an atanh series.
- Exponentials use range reduction, a Taylor series, and repeated squaring.
- Trig uses degree conversion when requested, modulo reduction, and Taylor series.
- Factorial uses an integer multiplication loop with a 0–170 domain.

The divmod loop is the fixed version documented in [Esolang's Brainfuck algorithms](https://esolangs.org/wiki/Brainfuck_algorithms#Fixed_Version). The generic multiplication loop and affine transfer loops are also conventional Brainfuck building blocks.

## Why it is fast enough

The interpreter folds pointer/increment runs and recognizes affine transfer loops, a repeated-addition multiplication loop, and the exact divmod loop. These are ordinary peephole optimizations of the Brainfuck source. They are not new language operators. Preconditions are checked; the two compound peepholes fall back to literal execution if the tape does not match their preconditions.

The tests compare complete tape states between literal execution and optimized execution for 324 multiplication cases, 950 division cases, and 512 signed addition/subtraction cases. A source mutation test confirms that replacing the final Brainfuck result changes the returned value. Numeric differential tests compare the emitted routines with Rust reference mathematics **in tests only**.

## Rebuild and run independently

From the app directory:

```sh
node brainfuck/generate.mjs
cargo test --locked
./build.sh
cargo run --release --bin bf-calc -- mul 128 4
cargo run --release --bin bf-calc -- sin 30
cargo run --release --bin bf-calc -- pow 9 0.5
```

The command-line runner uses the identical embedded Brainfuck programs without Qt or the expression parser. `build.sh` regenerates the committed programs before building. Changing a `.bf` file and running `cargo build` directly embeds that edited program; `build.sh` instead regenerates it from the assembler.

For expanded randomized, parser, VM and native-UI verification, see [the validation report](../validation/REPORT.md).
