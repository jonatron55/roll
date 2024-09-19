// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

mod ast;
mod eval;
mod graph;
mod lexer;
mod lookahead;
mod parser;
mod pp;

use std::{env, io::stdout, process::exit};

use parser::parse;
use pp::PP;

fn eval(mut arg: Option<String>, args: &mut impl Iterator<Item = String>) {
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
        Some(_) => eval::Evaluation::Rand(rand::thread_rng()),
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

    // Attempt to parse the input expression.
    let root = parse(input.as_str());

    if let Err(err) = root {
        eprintln!("\x1B[31m\x1B[1mError:\x1B[22m {err}\x1B[39m");
        exit(1);
    }

    let root = root.unwrap();

    // Echo the parsed expression.
    let mut stdout = stdout();
    let mut pp = PP::new(&mut stdout);
    root.accept(&mut pp).unwrap();
    println!();

    // Attempt to evaluate the parsed expression.
    let mut evaluator = eval::Evaluator::new(evaluation);
    let result = evaluator.eval(root.as_ref());

    match result {
        Ok(result) => {
            for roll in evaluator.rolls {
                print!("{} ", roll);
            }

            println!();
            println!("\x1B[2mtotal = \x1B[22m\x1B[1m{result}\x1B[22m");
        }
        Err(err) => {
            eprintln!("\x1B[31m\x1B[1mError:\x1B[22m {err}\x1B[39m");
            exit(1);
        }
    };
}

fn graph(lang: Option<String>, args: &mut impl Iterator<Item = String>) {
    let mut arg = args.next();
    let mut input = String::new();

    loop {
        input.push_str(arg.unwrap().as_str());
        input.push_str(" ");
        arg = args.next();
        if arg.is_none() {
            break;
        }
    }

    // Attempt to parse the input expression.
    let root = parse(input.as_str());

    if let Err(err) = root {
        eprintln!("\x1B[31m\x1B[1mError:\x1B[22m {err}\x1B[39m");
        exit(1);
    }

    let root = root.unwrap();

    // Echo the parsed expression.
    let mut stdout = stdout();
    let mut writer = match lang.as_deref() {
        Some("dot") => graph::GraphWriter::new_dot(&mut stdout),
        Some("mermaid") => graph::GraphWriter::new_mermaid(&mut stdout),
        _ => unreachable!(),
    };

    writer.write(root.as_ref()).unwrap();
}

fn main() {
    // The expression to evaluate is given on the command line and may be
    // preceded  by 'min', 'mid', or 'max' to specify the evaluation strategy.
    // The remaining arguments (or all arguments if no strategy is given) are
    // concatenated to form a single expression.
    let mut args = env::args().map(|arg| arg.to_lowercase());
    args.next();
    let arg = args.next();

    if let Some("dot" | "mermaid") = arg.as_deref() {
        graph(arg, &mut args);
    } else {
        eval(arg, &mut args);
    }
}
