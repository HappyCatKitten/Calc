use super::*;
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }
}
#[test]
fn generated_valid_expressions() {
    let mut rng = Rng(912441);
    for _ in 0..2000 {
        let a = (rng.next() % 2001) as f64 - 1000.;
        let b = (rng.next() % 501) as f64 + 1.;
        let c = (rng.next() % 51) as f64 + 1.;
        let expression = format!("(({a})+({b}))*({c})/({b})");
        let want = (a + b) * c / b;
        let got = evaluate(&expression, true).unwrap();
        assert!(
            (got - want).abs() < want.abs().max(1.) * 1e-11,
            "{expression}: {got}/{want}"
        )
    }
    println!("CAMPAIGN parser expressions: 2000 cases")
}
#[test]
fn malformed_input_never_panics() {
    let alphabet: Vec<char> = "0123456789.+-*/^%!()sqrtlnpie×÷−π abc🙂\0\n"
        .chars()
        .collect();
    let mut rng = Rng(718191);
    for _ in 0..10000 {
        let len = (rng.next() % 45) as usize;
        let text: String = (0..len)
            .map(|_| alphabet[(rng.next() as usize) % alphabet.len()])
            .collect();
        assert!(
            std::panic::catch_unwind(|| evaluate(&text, true)).is_ok(),
            "panic for {text:?}"
        )
    }
    println!("CAMPAIGN malformed parser: 10000 cases")
}
#[test]
fn memory_history_undo_sequences() {
    for i in 1..301 {
        let mut c = Calculator {
            transient: true,
            ..Calculator::new()
        };
        c.action(&format!("edit:{i}"));
        c.action("=");
        c.action("MS");
        c.action("edit:2");
        c.action("=");
        c.action("M+");
        c.action("C");
        c.action("MR");
        c.action("=");
        assert_eq!(c.result, (i + 2).to_string());
        c.action("×");
        c.action("3");
        c.action("=");
        assert_eq!(c.result, ((i + 2) * 3).to_string());
        c.action("C");
        c.action("reuse:0");
        c.action("=");
        assert_eq!(c.result, ((i + 2) * 3).to_string());
        c.action("clearHistory");
        assert!(c.history.is_empty());
    }
    println!("CAMPAIGN app action sequences: 300 cases")
}
#[test]
fn repeated_equals_edge_cases() {
    for (expr, want) in [
        ("5×−2", "20"),
        ("5÷−2", "1.25"),
        ("100+10%×2", "200.4"),
        ("100+10%+20%", "154"),
        ("2^3+4", "16"),
    ] {
        let mut c = Calculator {
            transient: true,
            ..Calculator::new()
        };
        c.action(&format!("edit:{expr}"));
        c.action("=");
        c.action("=");
        assert_eq!(c.result, want, "{expr}")
    }
}

#[test]
fn negative_current_operand() {
    for (expr, key, want) in [
        ("−4", "sqrt", None),
        ("5×−2", "square", Some("20")),
        ("−5", "±", Some("5")),
        ("e-2", "=", Some("-1.281718171541")),
    ] {
        let mut c = Calculator {
            transient: true,
            ..Calculator::new()
        };
        c.action(&format!("edit:{expr}"));
        c.action(key);
        c.action("=");
        if let Some(w) = want {
            assert_eq!(c.result, w, "{expr} {key}")
        } else {
            assert!(c.error, "{expr} {key}: {}", c.result)
        }
    }
}

#[test]
fn history_and_undo_are_bounded() {
    let mut c = Calculator {
        transient: true,
        ..Calculator::new()
    };
    for i in 0..350 {
        c.action(&format!("edit:{i}+1"));
        c.action("=");
    }
    assert_eq!(c.history.len(), 100);
    assert_eq!(c.history[0].result, "350");
    assert_eq!(c.history[99].result, "251");
    assert!(c.undo.len() <= 100);
    let before = c.result.clone();
    c.action("recall:9999");
    assert_eq!(c.result, before);
    c.action("delete:9999");
    assert_eq!(c.history.len(), 100);
    c.action("clearHistory");
    assert!(c.history.is_empty());
}
