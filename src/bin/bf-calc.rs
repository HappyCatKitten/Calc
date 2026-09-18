//! Run the exact Brainfuck routines embedded in the GUI, without Qt or its parser.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("Usage: bf-calc OP A [B] [--radians]\nOperations: add sub mul div neg abs sqrt ln log sin cos tan pow factorial pi e");
        std::process::exit(2);
    }
    let parse = |s: &str| -> f64 {
        s.parse().unwrap_or_else(|_| {
            eprintln!("Invalid numeric argument: {s}");
            std::process::exit(2)
        })
    };
    let a = parse(&args[1]);
    let b = args
        .get(2)
        .filter(|s| !s.starts_with("--"))
        .map(|s| parse(s))
        .unwrap_or(0.0);
    match calc_core::brainfuck::calculate(
        &args[0],
        a,
        b,
        !args.iter().any(|s| s == "--radians"),
    ) {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1)
        }
    }
}
