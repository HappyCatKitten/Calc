use crate::evaluate;
fn calculate(op: &str, a: f64, b: f64, degrees: bool) -> Result<f64, String> {
    let expression = match op {
        "add" => format!("({a})+({b})"),
        "sub" => format!("({a})-({b})"),
        "mul" => format!("({a})*({b})"),
        "div" => format!("({a})/({b})"),
        "pow" => format!("({a})^({b})"),
        "factorial" => format!("({a})!"),
        _ => format!("{op}({a})"),
    };
    evaluate(&expression, degrees)
}
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn f(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.f()
    }
}
fn check(failures: &mut Vec<String>, op: &str, a: f64, b: f64, degrees: bool, want: f64, tol: f64) {
    match calculate(op, a, b, degrees) {
        Ok(got) if (got - want).abs() <= tol => {}
        result => failures.push(format!(
            "{op}({a},{b}) deg={degrees}: {result:?}, wanted {want} tolerance {tol}"
        )),
    }
}
fn finish(name: &str, cases: usize, failures: Vec<String>) {
    println!(
        "CAMPAIGN {name}: {cases} cases, {} failures",
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "{}",
        failures
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
#[test]
fn randomized_arithmetic() {
    let mut rng = Rng(0x123456789abcdef);
    let mut failures = vec![];
    for _ in 0..3000 {
        let a = rng.range(-1e7, 1e7);
        let mut b = rng.range(-1e5, 1e5);
        if b.abs() < 0.001 {
            b = 0.125
        }
        for (op, want) in [
            ("add", a + b),
            ("sub", a - b),
            ("mul", a * b),
            ("div", a / b),
        ] {
            check(
                &mut failures,
                op,
                a,
                b,
                true,
                want,
                want.abs().max(1.) * 3e-13,
            )
        }
    }
    finish("random arithmetic", 12000, failures)
}
#[test]
fn randomized_scientific() {
    let mut rng = Rng(0xa65fe138793468);
    let mut failures = vec![];
    let mut count = 0;
    for _ in 0..600 {
        let x = 10f64.powf(rng.range(-16., 200.));
        for (op, want) in [("sqrt", x.sqrt()), ("ln", x.ln()), ("log", x.log10())] {
            check(
                &mut failures,
                op,
                x,
                0.,
                false,
                want,
                want.abs().max(1.) * 2e-8,
            );
            count += 1
        }
        let angle = rng.range(-10000., 10000.);
        for degrees in [false, true] {
            let x = if degrees { angle.to_radians() } else { angle };
            for (op, want) in [("sin", x.sin()), ("cos", x.cos())] {
                check(&mut failures, op, angle, 0., degrees, want, 2e-11);
                count += 1
            }
            if x.cos().abs() > 0.01 {
                check(
                    &mut failures,
                    "tan",
                    angle,
                    0.,
                    degrees,
                    x.tan(),
                    x.tan().abs().max(1.) * 1e-9,
                );
                count += 1
            }
        }
    }
    finish("scientific randomized", count, failures)
}
#[test]
fn randomized_powers() {
    let mut rng = Rng(0x7821634598);
    let mut failures = vec![];
    for _ in 0..600 {
        let a = rng.range(0.01, 100.);
        let b = rng.range(-8., 8.);
        let want = a.powf(b);
        check(
            &mut failures,
            "pow",
            a,
            b,
            true,
            want,
            want.abs().max(1.) * 5e-11,
        )
    }
    for n in 0..171 {
        let want = (1..=n).fold(1.0, |x, v| x * v as f64);
        check(
            &mut failures,
            "factorial",
            n as f64,
            0.,
            true,
            want,
            want.abs().max(1.) * 1e-12,
        )
    }
    finish("powers and factorials", 771, failures)
}
#[test]
fn domain_grid() {
    let mut count = 0;
    for n in 1..101 {
        for (op, a, b) in [
            ("sqrt", -(n as f64), 0.),
            ("ln", -(n as f64), 0.),
            ("log", -(n as f64), 0.),
            ("div", n as f64, 0.),
            ("factorial", n as f64 + 0.5, 0.),
            ("pow", -(n as f64), 0.5),
        ] {
            assert!(calculate(op, a, b, true).is_err(), "{op}({a},{b})");
            count += 1
        }
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(calculate("add", value, 1., true).is_err());
        count += 1
    }
    println!("CAMPAIGN domains: {count} cases")
}
#[test]
fn difficult_boundaries() {
    let mut failures = vec![];
    let mut count = 0;
    for angle in [-360., -270., -180., -90., 0., 90., 180., 270., 360.] {
        let want = (angle as f64).to_radians().sin();
        check(&mut failures, "sin", angle, 0., true, want, 1e-12);
        count += 1;
    }
    for x in [
        1e-24_f64,
        1e-23,
        1e-12,
        1e-6,
        0.999999999999,
        1.,
        1.000000000001,
        1e100,
        1e300,
    ] {
        for (op, want) in [("sqrt", (x as f64).sqrt()), ("ln", x.ln())] {
            check(
                &mut failures,
                op,
                x,
                0.,
                true,
                want,
                want.abs().max(1.) * 1e-10,
            );
            count += 1;
        }
    }
    finish("boundary values", count, failures)
}

#[test]
fn large_degree_angles_are_reduced_before_conversion() {
    for x in [1e15_f64, -1e15, 1e30, -1e30, 1e100, -1e100] {
        let reduced = (x % 360.0).to_radians();
        for (op, want) in [("sin", reduced.sin()), ("cos", reduced.cos())] {
            let got = calculate(op, x, 0.0, true).unwrap();
            assert!(
                (got - want).abs() < 1e-12,
                "{op}({x}) degrees: {got} vs {want}"
            );
        }
    }
}
