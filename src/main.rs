use std::env;
use std::io;
use rlox::{repl, run_file};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => {
            println!("Usage: lox [script]");
            std::process::exit(64)
        }
    }
}
