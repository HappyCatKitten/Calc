use super::*;
fn run(code: &str, opt: bool, initial: &[u32]) -> Result<Vec<BigUint>, String> {
    let mut tape = vec![BigUint::ZERO; 64];
    for (i, v) in initial.iter().enumerate() {
        tape[i] = BigUint::from(*v)
    }
    exec(
        &compile(code, opt)?,
        &mut tape,
        &mut 0,
        &mut std::iter::empty(),
        &mut vec![],
        &mut 1_000_000,
    )?;
    Ok(tape)
}
#[test]
fn exhaustive_small_divmod() {
    let mut n = 0;
    for a in 0..128 {
        for b in 1..65 {
            let initial = [a, b, 1];
            assert_eq!(
                run(DIV, true, &initial).unwrap(),
                run(DIV, false, &initial).unwrap()
            );
            n += 1
        }
    }
    println!("CAMPAIGN VM divmod: {n} cases")
}
#[test]
fn multiplication_nonzero_output() {
    let mut n = 0;
    for a in 0..32 {
        for b in 0..32 {
            for extra in [0, 1, 17] {
                assert_eq!(
                    run(MUL, true, &[a, b, extra]).unwrap(),
                    run(MUL, false, &[a, b, extra]).unwrap()
                );
                n += 1
            }
        }
    }
    println!("CAMPAIGN VM multiply: {n} cases")
}
#[test]
fn affine_transfer_variants() {
    let mut n = 0;
    for a in 0..80 {
        for b in 1..12 {
            let code = format!("[-<{}> {}]", "+".repeat(b as usize), "");
            let source = format!(">{code}");
            assert_eq!(
                run(&source, true, &[7, a]).unwrap(),
                run(&source, false, &[7, a]).unwrap()
            );
            n += 1
        }
    }
    println!("CAMPAIGN VM affine: {n} cases")
}
#[test]
fn boundaries_not_optimized_away() {
    for s in ["<>", "-+", ">-+<"] {
        assert!(run(s, false, &[]).is_err(), "literal {s}");
        assert!(run(s, true, &[]).is_err(), "optimized {s}");
    }
    let code = format!("{}+[><-]", ">".repeat(63));
    assert!(run(&code, true, &[]).is_err());
}
#[test]
fn zero_loop_has_no_side_effects() {
    for s in ["[<]", "[->-<]", "[----]", "[>+]"] {
        assert_eq!(run(s, true, &[]).unwrap(), run(s, false, &[]).unwrap())
    }
}
#[test]
fn fallback_preconditions() {
    for s in [MUL, DIV] {
        for init in [
            [0, 0, 0, 1, 0, 0],
            [0, 1, 0, 0, 0, 0],
            [2, 0, 1, 0, 0, 0],
            [1, 2, 1, 0, 0, 0],
        ] {
            let a = run(s, true, &init);
            let b = run(s, false, &init);
            assert_eq!(a, b, "{s} {init:?}")
        }
    }
}

#[test]
fn io_and_execution_budget() {
    for optimized in [false, true] {
        let mut tape = vec![BigUint::ZERO; 64];
        let mut output = vec![];
        exec(
            &compile(",[.,]", optimized).unwrap(),
            &mut tape,
            &mut 0,
            &mut b"Brainfuck 123\n".iter().copied(),
            &mut output,
            &mut 1000,
        )
        .unwrap();
        assert_eq!(output, b"Brainfuck 123\n");
        let error = exec(
            &compile("+[]", optimized).unwrap(),
            &mut tape,
            &mut 0,
            &mut std::iter::empty(),
            &mut vec![],
            &mut 100,
        )
        .unwrap_err();
        assert!(error.contains("execution limit"));
    }
}
#[test]
fn cell_growth_limits() {
    let mut tape = vec![BigUint::ZERO; 64];
    tape[0] = (BigUint::from(1u8) << LIMIT) - 1u8;
    assert!(exec(
        &compile("+", true).unwrap(),
        &mut tape,
        &mut 0,
        &mut std::iter::empty(),
        &mut vec![],
        &mut 100
    )
    .is_err());
    let mut tape = vec![BigUint::ZERO; 64];
    tape[0] = 1u8.into();
    tape[1] = 1u8.into();
    tape[2] = (BigUint::from(1u8) << LIMIT) - 1u8;
    assert!(exec(
        &compile(MUL, true).unwrap(),
        &mut tape,
        &mut 0,
        &mut std::iter::empty(),
        &mut vec![],
        &mut 100
    )
    .is_err());
}
