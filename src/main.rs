use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file.bf>", args[0]);
        std::process::exit(1);
    }

    let content = match fs::read_to_string(&args[1]) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    match bf_interpreter::run(&content, &[]) {
        Ok(output) => {
            io::stdout().write_all(&output).ok();
            io::stdout().flush().ok();
            println!();
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
