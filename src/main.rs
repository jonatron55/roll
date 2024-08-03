mod ast;
mod eval;
mod lexer;
mod lookahead;
mod parser;

use std::{env, process::exit};

use parser::parse;

fn main() {
    let mut args = env::args().map(|arg| arg.to_lowercase());
    args.next();
    let mut arg = args.next();

    let evaluation = match arg.as_deref() {
        Some("min") => {
            arg = args.next();
            eval::Evaluation::Min
        }
        Some("mid") => {
            arg = args.next();
            eval::Evaluation::Mid
        }
        Some("max") => {
            arg = args.next();
            eval::Evaluation::Max
        }
        Some(_) => eval::Evaluation::Random(rand::thread_rng()),
        None => exit(0),
    };

    let mut input = String::new();
    loop {
        input.push_str(arg.unwrap().as_str());
        input.push_str(" ");
        arg = args.next();
        if arg.is_none() {
            break;
        }
    }

    let root = parse(input.as_str());

    if let Err(err) = root {
        eprintln!("\x1B[31m\x1B[1mError:\x1B[22m {:?}\x1B[39m", err);
        exit(1);
    }

    let mut evaluator = eval::Evaluator::new(evaluation);
    let result = evaluator.eval(root.unwrap().as_ref());

    match result {
        Ok(result) => {
            for roll in evaluator.rolls {
                print!("{} ", roll);
            }

            println!();
            println!("\x1B[2mtotal = \x1B[22m\x1B[1m{}\x1B[22m", result);
        }
        Err(err) => {
            eprintln!("\x1B[31m\x1B[1mError:\x1B[22m {:?}\x1B[39m", err);
            exit(1);
        }
    };
}
