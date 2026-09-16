pub mod brainfuck;
use serde::{Deserialize, Serialize};
use std::{
    ffi::{c_char, CStr, CString},
    path::PathBuf,
};
#[derive(Clone, Default, Serialize, Deserialize)]
struct Entry {
    expression: String,
    result: String,
}
#[derive(Clone, Default)]
struct Snapshot {
    expression: String,
    result: String,
    done: bool,
    error: bool,
}
#[derive(Default, Serialize)]
pub struct Calculator {
    expression: String,
    result: String,
    history: Vec<Entry>,
    memory: Option<f64>,
    degrees: bool,
    error: bool,
    notice: String,
    #[serde(skip)]
    done: bool,
    #[serde(skip)]
    undo: Vec<Snapshot>,
    #[serde(skip)]
    redo: Vec<Snapshot>,
    #[serde(skip)]
    repeat: Option<(char, f64)>,
    #[serde(skip)]
    transient: bool,
}
fn history_path() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
        })
        .join("brainfuck-calculator/history.json")
}
fn format_number(n: f64) -> String {
    if n == 0.0 {
        return "0".into();
    }
    if n.abs() >= 1e15 || n.abs() < 1e-9 {
        return format!("{:.12e}", n);
    }
    if n.fract() == 0.0 {
        return format!("{n:.0}");
    }
    let decimals = (12 - n.abs().log10().floor() as i32).clamp(0, 20) as usize;
    format!("{n:.decimals$}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .into()
}
struct Parser {
    chars: Vec<char>,
    pos: usize,
    degrees: bool,
    depth: usize,
}
impl Parser {
    fn peek(&self) -> char {
        self.chars.get(self.pos).copied().unwrap_or('\0')
    }
    fn take(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expr(&mut self) -> Result<f64, String> {
        let mut a = self.term()?;
        loop {
            let op = self.peek();
            if op != '+' && op != '-' {
                break;
            }
            self.pos += 1;
            let start = self.pos;
            let mut b = self.term()?;
            // A standalone percentage after + or - is relative to the left subtotal.
            let slice = &self.chars[start..self.pos];
            if slice.last() == Some(&'%')
                && !slice.iter().any(|c| matches!(c, '*' | '/' | '^' | '('))
            {
                b = brainfuck::calculate("mul", b, a, self.degrees)?;
            }
            a = brainfuck::calculate(if op == '+' { "add" } else { "sub" }, a, b, self.degrees)?;
        }
        Ok(a)
    }
    fn term(&mut self) -> Result<f64, String> {
        let mut a = self.unary()?;
        loop {
            let op = self.peek();
            if op != '*' && op != '/' {
                break;
            }
            self.pos += 1;
            let b = self.unary()?;
            a = brainfuck::calculate(if op == '*' { "mul" } else { "div" }, a, b, self.degrees)?;
        }
        Ok(a)
    }
    fn unary(&mut self) -> Result<f64, String> {
        self.depth += 1;
        if self.depth > 80 {
            return Err("Expression too complex".into());
        }
        let r = if self.take('-') {
            self.unary()
                .and_then(|v| brainfuck::calculate("neg", v, 0.0, self.degrees))
        } else if self.take('+') {
            self.unary()
        } else {
            self.power()
        };
        self.depth -= 1;
        r
    }
    fn power(&mut self) -> Result<f64, String> {
        let a = self.postfix()?;
        if self.take('^') {
            brainfuck::calculate("pow", a, self.unary()?, self.degrees)
        } else {
            Ok(a)
        }
    }
    fn postfix(&mut self) -> Result<f64, String> {
        let mut a = self.atom()?;
        loop {
            if self.take('%') {
                a = brainfuck::calculate("div", a, 100.0, self.degrees)?
            } else if self.take('!') {
                a = brainfuck::calculate("factorial", a, 0.0, self.degrees)?;
            } else {
                break;
            }
        }
        Ok(a)
    }
    fn atom(&mut self) -> Result<f64, String> {
        if self.take('(') {
            let n = self.expr()?;
            if !self.take(')') {
                return Err("Missing closing parenthesis".into());
            }
            return Ok(n);
        }
        if self.peek().is_ascii_alphabetic() {
            let start = self.pos;
            while self.peek().is_ascii_alphabetic() {
                self.pos += 1
            }
            let name: String = self.chars[start..self.pos].iter().collect();
            if name == "pi" {
                return brainfuck::calculate("pi", 0.0, 0.0, self.degrees);
            }
            if name == "e" {
                return brainfuck::calculate("e", 0.0, 0.0, self.degrees);
            }
            if !self.take('(') {
                return Err("Function needs parentheses".into());
            }
            let v = self.unary_function_argument()?;
            return match name.as_str() {
                "sqrt" | "ln" | "log" | "sin" | "cos" | "tan" | "abs" => {
                    brainfuck::calculate(&name, v, 0.0, self.degrees)
                }
                _ => Err("Unknown function".into()),
            };
        }
        let start = self.pos;
        while self.peek().is_ascii_digit() || self.peek() == '.' {
            self.pos += 1
        }
        if self.pos == start {
            return Err("Incomplete expression".into());
        }
        if matches!(self.peek(), 'e' | 'E') {
            self.pos += 1;
            if matches!(self.peek(), '+' | '-') {
                self.pos += 1
            }
            while self.peek().is_ascii_digit() {
                self.pos += 1
            }
        }
        self.chars[start..self.pos]
            .iter()
            .collect::<String>()
            .parse()
            .map_err(|_| "Invalid number".into())
    }
    fn unary_function_argument(&mut self) -> Result<f64, String> {
        let v = self.expr()?;
        if !self.take(')') {
            return Err("Missing closing parenthesis".into());
        }
        Ok(v)
    }
}
fn evaluate(s: &str, degrees: bool) -> Result<f64, String> {
    if s.len() > 2048 {
        return Err("Expression too long".into());
    }
    let normalized = s
        .replace('×', "*")
        .replace('÷', "/")
        .replace('−', "-")
        .replace('π', "pi");
    let mut p = Parser {
        chars: normalized.chars().filter(|c| !c.is_whitespace()).collect(),
        pos: 0,
        degrees,
        depth: 0,
    };
    if p.chars.is_empty() {
        return Err("Enter a calculation".into());
    }
    let n = p.expr()?;
    if p.pos != p.chars.len() {
        return Err("Invalid expression".into());
    }
    if !n.is_finite() {
        return Err("Result outside supported range".into());
    }
    Ok(n)
}
fn binary_operator_at(text: &str, index: usize, operator: char) -> bool {
    if !matches!(operator, '+' | '-' | '−' | '*' | '/' | '×' | '÷' | '^') {
        return false;
    }
    if matches!(operator, '+' | '-' | '−') {
        let mut preceding = text[..index].chars().rev();
        match preceding.next() {
            None | Some('+' | '-' | '−' | '*' | '/' | '×' | '÷' | '^' | '(') => return false,
            Some('e' | 'E')
                if preceding
                    .next()
                    .map_or(false, |c| c.is_ascii_digit() || c == '.') =>
            {
                return false
            }
            _ => (),
        }
    }
    true
}
impl Calculator {
    fn new() -> Self {
        Self {
            result: "0".into(),
            degrees: true,
            ..Default::default()
        }
    }
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            expression: self.expression.clone(),
            result: self.result.clone(),
            done: self.done,
            error: self.error,
        }
    }
    fn restore(&mut self, s: Snapshot) {
        self.expression = s.expression;
        self.result = s.result;
        self.done = s.done;
        self.error = s.error;
        self.repeat = None;
    }
    fn save(&mut self) {
        if self.transient {
            return;
        }
        let p = history_path();
        let result = (|| -> std::io::Result<()> {
            std::fs::create_dir_all(p.parent().unwrap())?;
            let temp = p.with_extension("tmp");
            std::fs::write(&temp, serde_json::to_vec(&self.history)?)?;
            std::fs::rename(temp, p)
        })();
        if result.is_err() {
            self.notice = "Could not save history".into();
        }
    }
    fn preview(&mut self) {
        self.error = false;
        self.done = false;
        self.repeat = None;
        self.result = evaluate(&self.expression, self.degrees)
            .map(format_number)
            .unwrap_or_else(|_| "0".into());
    }
    fn operand_start(&self) -> usize {
        let mut depth = 0;
        for (i, c) in self.expression.char_indices().rev() {
            match c {
                ')' => depth += 1,
                '(' if depth > 0 => depth -= 1,
                _ if depth == 0 && binary_operator_at(&self.expression, i, c) => {
                    return i + c.len_utf8()
                }
                _ => (),
            }
        }
        0
    }
    fn action(&mut self, key: &str) {
        self.notice.clear();
        if key.is_empty() {
            return;
        }
        if key == "undo" {
            if let Some(s) = self.undo.pop() {
                self.redo.push(self.snapshot());
                self.restore(s)
            }
            return;
        }
        if key == "redo" {
            if let Some(s) = self.redo.pop() {
                self.undo.push(self.snapshot());
                self.restore(s)
            }
            return;
        }
        if key == "clearHistory" {
            self.history.clear();
            self.save();
            return;
        }
        if let Some(i) = key
            .strip_prefix("delete:")
            .and_then(|s| s.parse::<usize>().ok())
        {
            if i < self.history.len() {
                self.history.remove(i);
                self.save()
            }
            return;
        }
        if key == "deg" || key == "rad" {
            self.degrees = key == "deg";
            if !self.done {
                self.preview()
            }
            return;
        }
        if key == "MC" {
            self.memory = None;
            return;
        }
        if matches!(key, "MS" | "M+" | "M-") {
            if !self.error {
                if let Ok(v) = self.result.parse::<f64>() {
                    let n = match key {
                        "M+" => {
                            brainfuck::calculate("add", self.memory.unwrap_or(0.0), v, self.degrees)
                        }
                        "M-" => {
                            brainfuck::calculate("sub", self.memory.unwrap_or(0.0), v, self.degrees)
                        }
                        _ => Ok(v),
                    };
                    match n {
                        Ok(value) => self.memory = Some(value),
                        Err(error) => self.notice = error,
                    }
                }
            }
            return;
        }
        self.undo.push(self.snapshot());
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
        if let Some(i) = key
            .strip_prefix("recall:")
            .and_then(|s| s.parse::<usize>().ok())
        {
            if let Some(e) = self.history.get(i) {
                self.expression = e.result.clone();
                self.result = e.result.clone();
                self.done = true;
                self.error = false;
                self.repeat = None;
            }
            return;
        }
        if let Some(i) = key
            .strip_prefix("reuse:")
            .and_then(|s| s.parse::<usize>().ok())
        {
            if let Some(e) = self.history.get(i) {
                self.expression = e.expression.clone();
                self.preview();
            }
            return;
        }
        if let Some(s) = key.strip_prefix("edit:") {
            self.expression = s.chars().take(1024).collect();
            self.preview();
            return;
        }
        match key {
            "C" => {
                self.expression.clear();
                self.result = "0".into();
                self.done = false;
                self.error = false;
                self.repeat = None;
            }
            "CE" => {
                let start = self.operand_start();
                self.expression.truncate(start);
                self.preview()
            }
            "back" => {
                self.expression.pop();
                self.preview();
            }
            "MR" => {
                if let Some(n) = self.memory {
                    let s = format_number(n);
                    if self.done || self.error {
                        self.expression = s
                    } else {
                        let start = self.operand_start();
                        self.expression.truncate(start);
                        self.expression.push_str(&s)
                    }
                    self.preview()
                }
            }
            "=" => {
                if self.expression.is_empty() {
                    return;
                }
                if self.done {
                    if let Some((op, n)) = self.repeat {
                        self.expression = format!("{}{}{}", self.result, op, format_number(n))
                    } else {
                        return;
                    }
                }
                match evaluate(&self.expression, self.degrees) {
                    Ok(n) => {
                        if !self.done {
                            self.repeat = None;
                            let mut depth = 0;
                            for (i, c) in self.expression.char_indices().rev() {
                                if c == ')' {
                                    depth += 1
                                } else if c == '(' {
                                    depth -= 1
                                } else if depth == 0
                                    && matches!(c, '+' | '−' | '×' | '÷' | '*' | '/' | '-')
                                    && i > 0
                                {
                                    if binary_operator_at(&self.expression, i, c) {
                                        if let Ok(rhs) = evaluate(
                                            &self.expression[i + c.len_utf8()..],
                                            self.degrees,
                                        ) {
                                            let relative = matches!(c, '+' | '-' | '−')
                                                && self.expression[i + c.len_utf8()..]
                                                    .ends_with('%');
                                            let rhs = if relative {
                                                brainfuck::calculate(
                                                    "mul",
                                                    rhs,
                                                    evaluate(&self.expression[..i], self.degrees)
                                                        .unwrap_or(1.0),
                                                    self.degrees,
                                                )
                                                .unwrap_or(rhs)
                                            } else {
                                                rhs
                                            };
                                            self.repeat = Some((c, rhs));
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        self.result = format_number(n);
                        self.history.insert(
                            0,
                            Entry {
                                expression: self.expression.clone(),
                                result: self.result.clone(),
                            },
                        );
                        self.history.truncate(100);
                        self.save();
                        self.done = true;
                        self.error = false;
                    }
                    Err(e) => {
                        self.result = e;
                        self.error = true;
                        self.done = false;
                    }
                }
            }
            "±" | "square" | "sqrt" | "reciprocal" | "sin" | "cos" | "tan" | "ln" | "log"
            | "abs" => {
                if self.error {
                    return;
                }
                if self.done {
                    self.expression = self.result.clone();
                }
                let start = self.operand_start();
                let part = self.expression[start..].to_string();
                let part = if part.is_empty() { "0" } else { &part };
                let wrapped = match key {
                    "±" => format!("-({part})"),
                    "square" => format!("({part})^2"),
                    "reciprocal" => format!("(1/({part}))"),
                    _ => format!("{key}({part})"),
                };
                self.expression.truncate(start);
                self.expression.push_str(&wrapped);
                self.preview();
            }
            "+" | "−" | "×" | "÷" | "^" => {
                if self.error {
                    return;
                }
                if self.done {
                    self.expression = self.result.clone();
                    self.done = false;
                    self.repeat = None;
                }
                if self.expression.is_empty() {
                    if key == "−" {
                        self.expression.push('−')
                    }
                    return;
                }
                if key == "−" && self.expression.ends_with(['×', '÷', '^']) {
                    self.expression.push('−');
                    return;
                }
                if self.expression.ends_with(['+', '−', '×', '÷', '^']) {
                    self.expression.pop();
                }
                self.expression.push_str(key);
            }
            _ => {
                if !(key.len() == 1 && "0123456789.()%!".contains(key) || key == "pi" || key == "e")
                {
                    return;
                }
                if self.done || self.error {
                    if matches!(key, "%" | "!") {
                        self.expression = self.result.clone()
                    } else {
                        self.expression.clear()
                    }
                    self.done = false;
                    self.error = false;
                }
                if self.expression.len() > 1024 {
                    return;
                }
                if key == "." {
                    let last = self
                        .expression
                        .rsplit(['+', '−', '×', '÷', '^', '(', ')'])
                        .next()
                        .unwrap_or("");
                    if last.contains('.') {
                        return;
                    }
                    if last.is_empty() {
                        self.expression.push('0')
                    }
                }
                if matches!(key, "pi" | "e" | "(")
                    && self.expression.chars().last().map_or(false, |c| {
                        c.is_ascii_digit() || matches!(c, ')' | '%' | '!')
                    })
                {
                    self.expression.push('×');
                }
                self.expression.push_str(key);
                self.preview();
            }
        }
    }
}
#[no_mangle]
pub extern "C" fn calc_new() -> *mut Calculator {
    let history = std::fs::read(history_path()).or_else(|_| { let old=history_path().parent().unwrap().parent().unwrap().join("obsidian-calculator/history.json"); std::fs::read(old) })
        .ok()
        .and_then(|s| serde_json::from_slice(&s).ok())
        .unwrap_or_default();
    Box::into_raw(Box::new(Calculator {
        history,
        ..Calculator::new()
    }))
}
#[no_mangle]
pub unsafe extern "C" fn calc_free(p: *mut Calculator) {
    if !p.is_null() {
        drop(Box::from_raw(p));
    }
}
#[no_mangle]
pub unsafe extern "C" fn calc_action(p: *mut Calculator, k: *const c_char) -> *mut c_char {
    let c = &mut *p;
    if let Ok(k) = CStr::from_ptr(k).to_str() {
        c.action(k)
    }
    CString::new(serde_json::to_string(c).unwrap())
        .unwrap()
        .into_raw()
}
#[no_mangle]
pub unsafe extern "C" fn calc_string_free(p: *mut c_char) {
    if !p.is_null() {
        drop(CString::from_raw(p));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn calc() -> Calculator {
        Calculator {
            transient: true,
            ..Calculator::new()
        }
    }
    fn equal(s: &str, n: f64) {
        let actual = evaluate(s, true).unwrap();
        assert!((actual - n).abs() < 1e-9, "{s}: {actual} != {n}")
    }
    #[test]
    fn basic() {
        for (s, n) in [
            ("128×4", 512.),
            ("2+3×4", 14.),
            ("(2+3)*4", 20.),
            ("-5*2", -10.),
            ("1.5/.5", 3.),
            ("0.1+0.2", 0.3),
        ] {
            equal(s, n)
        }
    }
    #[test]
    fn scientific() {
        for (s, n) in [
            ("2^3^2", 512.),
            ("-2^2", -4.),
            ("2^-2", 0.25),
            ("sqrt(81)", 9.),
            ("5!", 120.),
            ("sin(30)", 0.5),
            ("log(100)", 2.),
            ("ln(e)", 1.),
        ] {
            equal(s, n)
        }
        assert!((evaluate("sin(pi/2)", false).unwrap() - 1.).abs() < 1e-12)
    }
    #[test]
    fn percent() {
        for (s, n) in [
            ("200+10%", 220.),
            ("200-10%", 180.),
            ("200*10%", 20.),
            ("50%", 0.5),
            ("200/10%", 2000.),
        ] {
            equal(s, n)
        }
    }
    #[test]
    fn invalid() {
        for s in [
            "1/0", "2+", ".", "2**3", "(2+3", "sqrt(-1)", "log(0)", "171!", "2.5!", "tan(90)",
            "2foo", "", "1e999",
        ] {
            assert!(evaluate(s, true).is_err(), "{s}")
        }
    }
    #[test]
    fn grouping_not_in_engine() {
        assert!(evaluate("1,000", true).is_err());
        equal("1e-5*100000", 1.)
    }
    #[test]
    fn format() {
        assert_eq!(format_number(0.1 + 0.2), "0.3");
        assert_eq!(format_number(512.), "512")
    }
    #[test]
    fn repeat_and_continue() {
        let mut c = calc();
        for k in ["edit:2+3", "=", "="] {
            c.action(k)
        }
        assert_eq!(c.result, "8");
        c.action("×");
        c.action("2");
        c.action("=");
        assert_eq!(c.result, "16");
        c.action("7");
        assert_eq!(c.expression, "7")
    }
    #[test]
    fn undo_and_clear_entry() {
        let mut c = calc();
        c.action("edit:12+34");
        c.action("CE");
        assert_eq!(c.expression, "12+");
        c.action("undo");
        assert_eq!(c.expression, "12+34");
        c.action("redo");
        assert_eq!(c.expression, "12+")
    }
    #[test]
    fn memory() {
        let mut c = calc();
        for k in ["edit:50", "MS", "edit:25", "M+", "C", "MR"] {
            c.action(k)
        }
        assert_eq!(c.result, "75");
        c.action("M-");
        assert_eq!(c.memory, Some(0.));
        c.action("MC");
        assert!(c.memory.is_none())
    }
    #[test]
    fn history() {
        let mut c = calc();
        for k in ["edit:5*4", "=", "C", "reuse:0"] {
            c.action(k)
        }
        assert_eq!(c.expression, "5*4");
        c.action("recall:0");
        assert_eq!(c.result, "20");
        c.action("delete:0");
        assert!(c.history.is_empty())
    }
    #[test]
    fn unary() {
        let mut c = calc();
        for k in ["edit:100+9", "sqrt", "="] {
            c.action(k)
        }
        assert_eq!(c.result, "103");
        c.action("±");
        c.action("=");
        assert_eq!(c.result, "-103")
    }
    #[test]
    fn errors_recover() {
        let mut c = calc();
        for k in ["edit:1/0", "=", "back", "2", "="] {
            c.action(k)
        }
        assert_eq!(c.result, "0.5");
        assert!(!c.error)
    }
    #[test]
    fn fractional_large_number() {
        assert_eq!(format_number(1234567.89 * 2.0), "2469135.78");
        assert_eq!(format_number(1000.0), "1000");
    }
    #[test]
    fn repeated_percent() {
        let mut c = calc();
        for k in ["edit:200+10%", "=", "="] {
            c.action(k)
        }
        assert_eq!(c.result, "240");
    }
    #[test]
    fn reciprocal_operand() {
        let mut c = calc();
        for k in ["edit:100/4", "reciprocal", "="] {
            c.action(k)
        }
        assert_eq!(c.result, "400");
    }
    #[test]
    fn signed_keyboard_operand() {
        let mut c = calc();
        for k in ["5", "×", "−", "2", "="] {
            c.action(k)
        }
        assert_eq!(c.result, "-10");
    }
    #[test]
    fn constants_multiply() {
        let mut c = calc();
        for k in ["2", "pi", "="] {
            c.action(k)
        }
        assert!((c.result.parse::<f64>().unwrap() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
    }
    #[test]
    fn depth_limit() {
        assert!(evaluate(&format!("{}1{}", "(".repeat(100), ")".repeat(100)), true).is_err())
    }
}

#[cfg(test)]
#[path = "../tests/stress/numeric.rs"]
mod stress_numeric;
#[cfg(test)]
#[path = "../tests/stress/parser.rs"]
mod stress_parser;
