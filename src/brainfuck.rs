//! Generic optimizing Brainfuck VM and tape ABI. No expression evaluation here.
//! The only specializations are semantics-preserving loop peepholes, also checked
//! against the literal interpreter in tests. Numeric algorithms live in *.bf.
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use std::{collections::BTreeMap, sync::OnceLock};
const MUL: &str = "[>[->+>+<<]>>[-<<+>>]<<<-]";
const DIV: &str = "[->-[>+>>]>[[-<+>]+>+>>]<<<<<]";
const LIMIT: usize = 4096;
const PRECISION: usize = 24;
#[derive(Debug)]
enum Op {
    Move(isize),
    Add(i32),
    Affine {
        terms: Vec<(isize, i32)>,
        low: isize,
        high: isize,
    },
    Mul,
    Div,
    In,
    Out,
    Open(usize),
    Close(usize),
}
fn affine(s: &[u8]) -> Option<(usize, Vec<(isize, i32)>, isize, isize)> {
    let mut p = 0isize;
    let mut low = 0;
    let mut high = 0;
    let mut terms: BTreeMap<isize, i32> = BTreeMap::new();
    for (i, &c) in s.iter().enumerate().skip(1) {
        match c {
            b'>' => {
                p += 1;
                high = high.max(p)
            }
            b'<' => {
                p -= 1;
                low = low.min(p)
            }
            b'+' | b'-' => {
                let delta = if c == b'+' { 1 } else { -1 };
                let prior = terms.entry(p).or_insert(0);
                // Mixed directions could hide an intermediate underflow.
                if *prior != 0 && prior.signum() != delta {
                    return None;
                }
                *prior += delta;
            }
            b']' => {
                if p != 0 || terms.remove(&0) != Some(-1) {
                    return None;
                }
                return Some((i + 1, terms.into_iter().collect(), low, high));
            }
            _ => return None,
        }
    }
    None
}

fn compile(source: &str, optimize: bool) -> Result<Vec<Op>, String> {
    let s: Vec<u8> = source.bytes().filter(|c| b"><+-[],.".contains(c)).collect();
    let mut result = Vec::new();
    let mut stack = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if optimize && s[i..].starts_with(MUL.as_bytes()) {
            result.push(Op::Mul);
            i += MUL.len();
            continue;
        }
        if optimize && s[i..].starts_with(DIV.as_bytes()) {
            result.push(Op::Div);
            i += DIV.len();
            continue;
        }
        if optimize && s[i] == b'[' {
            if let Some((len, terms, low, high)) = affine(&s[i..]) {
                result.push(Op::Affine { terms, low, high });
                i += len;
                continue;
            }
        }
        match s[i] {
            b'>' | b'<' => {
                let mut n = 0;
                let start = i;
                let c = s[i];
                while i < s.len() && s[i] == c && (optimize || i == start) {
                    n += if s[i] == b'>' { 1 } else { -1 };
                    i += 1
                }
                result.push(Op::Move(n));
                continue;
            }
            b'+' | b'-' => {
                let mut n = 0;
                let start = i;
                let c = s[i];
                while i < s.len() && s[i] == c && (optimize || i == start) {
                    n += if s[i] == b'+' { 1 } else { -1 };
                    i += 1
                }
                result.push(Op::Add(n));
                continue;
            }
            b'[' => {
                stack.push(result.len());
                result.push(Op::Open(0))
            }
            b']' => {
                let open = stack.pop().ok_or("Unmatched Brainfuck bracket")?;
                let end = result.len();
                result[open] = Op::Open(end + 1);
                result.push(Op::Close(open + 1))
            }
            b',' => result.push(Op::In),
            b'.' => result.push(Op::Out),
            _ => unreachable!(),
        }
        i += 1;
    }
    if !stack.is_empty() {
        return Err("Unmatched Brainfuck bracket".into());
    }
    Ok(result)
}
fn index(p: usize, offset: isize, len: usize) -> Result<usize, String> {
    let n = p as isize + offset;
    if n < 0 || n as usize >= len {
        Err("Brainfuck tape boundary".into())
    } else {
        Ok(n as usize)
    }
}
fn change(value: &mut BigUint, delta: i32) -> Result<(), String> {
    if delta >= 0 {
        *value += delta as u32
    } else {
        let n = BigUint::from(delta.unsigned_abs());
        if *value < n {
            return Err("Brainfuck cell underflow".into());
        }
        *value -= n
    }
    if value.bits() as usize > LIMIT {
        return Err("Result outside supported range".into());
    }
    Ok(())
}
fn exec(
    ops: &[Op],
    tape: &mut [BigUint],
    pointer: &mut usize,
    input: &mut impl Iterator<Item = u8>,
    output: &mut Vec<u8>,
    fuel: &mut usize,
) -> Result<(), String> {
    let mut pc = 0;
    while pc < ops.len() {
        if *fuel == 0 {
            return Err("Calculation exceeded Brainfuck execution limit".into());
        }
        *fuel -= 1;
        let p = *pointer;
        match &ops[pc] {
            Op::Move(n) => *pointer = index(p, *n, tape.len())?,
            Op::Add(n) => change(&mut tape[p], *n)?,
            Op::Affine { terms, low, high } => {
                let n = tape[p].clone();
                if n.is_zero() {
                    pc += 1;
                    continue;
                }
                index(p, *low, tape.len())?;
                index(p, *high, tape.len())?;
                for &(offset, k) in terms {
                    let at = index(p, offset, tape.len())?;
                    let amount = &n * k.unsigned_abs();
                    if k > 0 {
                        tape[at] += &amount
                    } else {
                        if tape[at] < amount {
                            return Err("Brainfuck cell underflow".into());
                        }
                        tape[at] -= &amount
                    }
                    if tape[at].bits() as usize > LIMIT {
                        return Err("Result outside supported range".into());
                    }
                }
                tape[p] = BigUint::ZERO;
            }
            Op::Mul => {
                if tape[p].is_zero() {
                    pc += 1;
                    continue;
                }
                index(p, 3, tape.len())?;
                if tape[p + 3].is_zero() {
                    let amount = &tape[p] * &tape[p + 1];
                    if amount.bits() as usize > LIMIT {
                        return Err("Result outside supported range".into());
                    }
                    tape[p + 2] += amount;
                    if tape[p + 2].bits() as usize > LIMIT {
                        return Err("Result outside supported range".into());
                    }
                    tape[p] = BigUint::ZERO;
                } else {
                    exec(&compile(MUL, false)?, tape, pointer, input, output, fuel)?;
                }
            }
            Op::Div => {
                if tape[p].is_zero() {
                    pc += 1;
                    continue;
                }
                index(p, 5, tape.len())?;
                if !tape[p + 1].is_zero()
                    && tape[p + 2] == BigUint::from(1u8)
                    && tape[p + 3..p + 6].iter().all(Zero::is_zero)
                {
                    let n = tape[p].clone();
                    let d = tape[p + 1].clone();
                    let q = &n / &d;
                    let r = &n % &d;
                    tape[p] = BigUint::ZERO;
                    tape[p + 1] = &d - &r;
                    tape[p + 2] = r + 1u8;
                    tape[p + 3] = q;
                } else {
                    exec(&compile(DIV, false)?, tape, pointer, input, output, fuel)?;
                }
            }
            Op::In => tape[p] = BigUint::from(input.next().unwrap_or(0)),
            Op::Out => output.push((&tape[p] % 256u32).to_u8().unwrap()),
            Op::Open(end) => {
                if tape[p].is_zero() {
                    pc = *end;
                    continue;
                }
            }
            Op::Close(start) => {
                if !tape[p].is_zero() {
                    pc = *start;
                    continue;
                }
            }
        }
        pc += 1;
    }
    Ok(())
}
const PROGRAMS: [(&str, &str); 16] = [
    ("add", include_str!("../brainfuck/add.bf")),
    ("sub", include_str!("../brainfuck/sub.bf")),
    ("mul", include_str!("../brainfuck/mul.bf")),
    ("div", include_str!("../brainfuck/div.bf")),
    ("neg", include_str!("../brainfuck/neg.bf")),
    ("abs", include_str!("../brainfuck/abs.bf")),
    ("sqrt", include_str!("../brainfuck/sqrt.bf")),
    ("ln", include_str!("../brainfuck/ln.bf")),
    ("log", include_str!("../brainfuck/log.bf")),
    ("sin", include_str!("../brainfuck/sin.bf")),
    ("cos", include_str!("../brainfuck/cos.bf")),
    ("tan", include_str!("../brainfuck/tan.bf")),
    ("pow", include_str!("../brainfuck/pow.bf")),
    ("factorial", include_str!("../brainfuck/factorial.bf")),
    ("pi", include_str!("../brainfuck/pi.bf")),
    ("e", include_str!("../brainfuck/e.bf")),
];
static COMPILED: OnceLock<Vec<Vec<Op>>> = OnceLock::new();
fn encode(n: f64) -> Result<BigUint, String> {
    if !n.is_finite() {
        return Err("Result outside supported range".into());
    }
    // Decimal serialization only; the Brainfuck programs perform numeric operations.
    let decimal = format!("{:.PRECISION$}", n.abs());
    let magnitude = decimal
        .replace('.', "")
        .parse::<BigUint>()
        .map_err(|_| "Invalid input number")?;
    if n != 0.0 && magnitude.is_zero() {
        return Err("Brainfuck precision limit: minimum magnitude is 1e-24".into());
    }
    Ok(magnitude)
}
fn decode(n: &BigUint, negative: bool) -> Result<f64, String> {
    let mut s = n.to_string();
    if s.len() <= PRECISION {
        s = format!("{:0>width$}", s, width = PRECISION + 1)
    }
    s.insert(s.len() - PRECISION, '.');
    if negative && !n.is_zero() {
        s.insert(0, '-');
    }
    let result = s.parse::<f64>().map_err(|_| "Invalid Brainfuck result")?;
    if !result.is_finite() {
        return Err("Result outside supported range".into());
    }
    Ok(result)
}
pub fn calculate(name: &str, a: f64, b: f64, degrees: bool) -> Result<f64, String> {
    let pos = PROGRAMS
        .iter()
        .position(|(n, _)| *n == name)
        .ok_or("Unknown Brainfuck routine")?;
    let ops = COMPILED.get_or_init(|| {
        PROGRAMS
            .iter()
            .map(|(_, s)| compile(s, true).expect("checked BF source"))
            .collect()
    });
    let mut tape = vec![BigUint::ZERO; 512];
    tape[0] = encode(a)?;
    tape[1] = BigUint::from(u8::from(a.is_sign_negative()));
    tape[2] = encode(b)?;
    tape[3] = BigUint::from(u8::from(b.is_sign_negative()));
    tape[4] = BigUint::from(u8::from(degrees));
    exec(
        &ops[pos],
        &mut tape,
        &mut 0,
        &mut std::iter::empty(),
        &mut Vec::new(),
        &mut 2_000_000,
    )?;
    if !tape[5].is_zero() {
        return Err(match tape[5].to_u32().unwrap_or(3) {
            1 => "Cannot divide by zero",
            2 => "Function input is outside its domain",
            4 => "Factorial needs an integer from 0 to 170",
            5 => "Tangent is undefined at this angle",
            6 => "Radian angle exceeds the reliable range of 1e12",
            _ => "Result outside supported range",
        }
        .into());
    }
    if tape[6].is_zero() && a != 0.0 && (matches!(name, "mul" | "div") && b != 0.0 || name == "pow")
    {
        return Err("Result below Brainfuck precision of 1e-24".into());
    }
    decode(&tape[6], !tape[7].is_zero())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn run(source: &str, opt: bool, values: &[u32]) -> Result<Vec<BigUint>, String> {
        let mut tape = vec![BigUint::ZERO; 512];
        for (i, v) in values.iter().enumerate() {
            tape[i] = (*v).into()
        }
        exec(
            &compile(source, opt)?,
            &mut tape,
            &mut 0,
            &mut std::iter::empty(),
            &mut Vec::new(),
            &mut 5_000_000,
        )?;
        Ok(tape)
    }
    #[test]
    fn literal_matches_multiplication() {
        for a in 0..18 {
            for b in 0..18 {
                assert_eq!(
                    run(MUL, true, &[a, b]).unwrap(),
                    run(MUL, false, &[a, b]).unwrap()
                )
            }
        }
    }
    #[test]
    fn literal_matches_division() {
        for a in 0..50 {
            for b in 1..20 {
                assert_eq!(
                    run(DIV, true, &[a, b, 1]).unwrap(),
                    run(DIV, false, &[a, b, 1]).unwrap()
                )
            }
        }
    }
    #[test]
    fn literal_matches_add_sub() {
        for op in ["add", "sub"] {
            let source = PROGRAMS.iter().find(|(n, _)| *n == op).unwrap().1;
            for a in 0..8 {
                for b in 0..8 {
                    for sa in 0..2 {
                        for sb in 0..2 {
                            let v = [a, sa, b, sb];
                            assert_eq!(
                                run(source, true, &v).unwrap(),
                                run(source, false, &v).unwrap(),
                                "{op} {v:?}"
                            )
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn all_sources_are_pure() {
        for (_, s) in PROGRAMS {
            assert!(s.bytes().all(|c| b"><+-[],.\n".contains(&c)));
            compile(s, true).unwrap();
        }
    }
    #[test]
    fn arithmetic() {
        for (op, a, b, want) in [
            ("add", 2., 3., 5.),
            ("sub", 2., 3., -1.),
            ("mul", -1.5, 4., -6.),
            ("div", 5., 2., 2.5),
            ("sqrt", 81., 0., 9.),
            ("pow", 2., 10., 1024.),
            ("factorial", 5., 0., 120.),
        ] {
            let n = calculate(op, a, b, true).unwrap();
            assert!((n - want).abs() < 1e-10, "{op}: {n}");
        }
    }
    #[test]
    fn scientific() {
        for (op, x, want) in [
            ("sin", 30., 0.5),
            ("cos", 60., 0.5),
            ("tan", 45., 1.),
            ("ln", 2., std::f64::consts::LN_2),
            ("log", 100., 2.),
        ] {
            let n = calculate(op, x, 0., true).unwrap();
            assert!((n - want).abs() < 1e-10, "{op}: {n}")
        }
        assert!((calculate("pow", 9., 0.5, true).unwrap() - 3.).abs() < 1e-10);
    }
    #[test]
    fn resource_and_domain_guards() {
        for (op, a, b) in [
            ("div", 1., 0.),
            ("sqrt", -1., 0.),
            ("log", 0., 0.),
            ("tan", 90., 0.),
            ("pow", -2., 0.5),
            ("factorial", 171., 0.),
            ("pow", 10., 10000.),
        ] {
            assert!(calculate(op, a, b, true).is_err(), "{op}")
        }
    }
    #[test]
    fn differential_numeric_grid() {
        for a in [-1000.25_f64, -12.5, -1.0, 0.0, 0.125, 1.0, 25.5, 10000.25] {
            for b in [-15.25_f64, -1.0, 0.125, 1.0, 10.5, 1000.0] {
                for (op, expected) in [
                    ("add", a + b),
                    ("sub", a - b),
                    ("mul", a * b),
                    ("div", a / b),
                ] {
                    let actual = calculate(op, a, b, true).unwrap();
                    assert!(
                        (actual - expected).abs() <= expected.abs().max(1.0) * 1e-12,
                        "{op} {a} {b}: {actual} vs {expected}"
                    );
                }
            }
        }
    }
    #[test]
    fn scientific_range_regressions() {
        for x in [1e-20_f64, 0.001, 0.25, 1.0, 3.5, 10000.0, 1e100] {
            for (op, expected) in [("ln", x.ln()), ("log", x.log10()), ("sqrt", x.sqrt())] {
                let actual = calculate(op, x, 0.0, true).unwrap();
                assert!(
                    (actual - expected).abs() <= expected.abs().max(1.0) * 1e-11,
                    "{op} {x}: {actual} vs {expected}"
                );
            }
        }
        for x in [
            -720.0_f64, -180.0, -45.0, 0.0, 30.0, 90.0, 180.0, 360.0, 720.0,
        ] {
            for (op, expected) in [("sin", x.to_radians().sin()), ("cos", x.to_radians().cos())] {
                assert!(
                    (calculate(op, x, 0.0, true).unwrap() - expected).abs() < 1e-12,
                    "{op} {x}"
                );
            }
        }
        for (a, b) in [
            (0.25_f64, 0.5_f64),
            (2.0, -3.0),
            (-2.0, 3.0),
            (10.0, 2.5),
            (100.0, 0.25),
        ] {
            let actual = calculate("pow", a, b, true).unwrap();
            assert!((actual - a.powf(b)).abs() < 1e-9, "pow {a} {b}: {actual}");
        }
    }
    #[test]
    fn source_changes_change_results() {
        let source = PROGRAMS[0].1;
        assert_eq!(
            run(source, true, &[2, 0, 3, 0]).unwrap()[6],
            BigUint::from(5u8)
        );
        assert_eq!(
            run(&(source.to_owned() + "[-]"), true, &[2, 0, 3, 0]).unwrap()[6],
            BigUint::ZERO
        );
    }
    #[test]
    fn explicit_underflow() {
        assert!(calculate("mul", 1e-20, 1e-20, true).is_err());
        assert!(calculate("div", 1e-20, 1e20, true).is_err());
    }
    #[test]
    fn no_native_fallback() {
        assert!(calculate("unknown", 1., 2., true).is_err());
        assert!(calculate("add", 1e-30, 0., true).is_err());
        assert!(run("<", true, &[]).is_err());
        assert!(compile("[", true).is_err());
    }
}

#[cfg(test)]
#[path = "../tests/stress/vm.rs"]
mod stress_vm;
