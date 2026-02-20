// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

mod ast;
mod eval;
mod graph;
mod help;
mod lexer;
mod lookahead;
mod parser;
mod pp;

#[cfg(test)]
mod tests;

use std::{env, fmt::Display, io::stdout, process::exit};

use anstream::{eprintln, print, println};
use anstyle::{AnsiColor, Color, Style};

use crate::{eval::DieRoll, help::print_help, parser::parse};

const ERROR: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Red)))
    .bold();
const KEEP: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Green)))
    .dimmed();
const KEEP_DIE: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Green)))
    .italic();
const KEEP_VAL: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::BrightGreen)))
    .bold();
const DROP: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Yellow)))
    .dimmed()
    .strikethrough();
const DROP_DIE: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Yellow)))
    .strikethrough()
    .italic();
const DROP_VAL: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
    .bold()
    .strikethrough();
const TOTAL: Style = Style::new().dimmed().italic();
const TOTAL_VAL: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::BrightWhite)))
    .bold();

fn ok_or_exit<T, E>(result: Result<T, E>) -> T
where
    E: Display,
{
    match result {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{ERROR}Error:{ERROR:#} {err}");
            exit(1);
        }
    }
}

fn some_or_exit<T>(option: Option<T>, msg: &str) -> T {
    match option {
        Some(value) => value,
        None => {
            eprintln!("{ERROR}Error:{ERROR:#} {msg}");
            exit(0)
        }
    }
}

fn eval(mut arg: Option<String>, args: &mut impl Iterator<Item = String>) {
    let evaluation = match arg.as_deref() {
        Some("help") | Some("-help") | Some("--help") | Some("/help") | Some("-h") | Some("/h")
        | Some("-?") | Some("/?") | Some("?") | None => {
            print_help();
            exit(0);
        }
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
        Some(_) => eval::Evaluation::Rand(rand::rng()),
    };

    let mut input = String::new();
    loop {
        input.push_str(some_or_exit(arg, "missing expression").as_str());
        input.push_str(" ");
        arg = args.next();
        if arg.is_none() {
            break;
        }
    }

    // Attempt to parse the input expression.
    let root = parse(input.as_str());
    let root = ok_or_exit(root);

    // Echo the parsed expression.
    println!("{root}");

    // Attempt to evaluate the parsed expression.
    let mut evaluator = eval::Evaluator::new(evaluation);
    let result = evaluator.eval(root.as_ref());

    match result {
        Ok(result) => {
            for roll in evaluator.rolls {
                print_roll(&roll);
                print!(" ");
            }

            println!("\n{TOTAL}Total:{TOTAL:#} {TOTAL_VAL}{result}{TOTAL_VAL:#}");
        }
        Err(err) => {
            eprintln!("{ERROR}Error:{ERROR:#} {err}");
            exit(1);
        }
    };
}

fn graph(lang: Option<String>, args: &mut impl Iterator<Item = String>) {
    let mut arg = args.next();
    let mut input = String::new();

    loop {
        input.push_str(some_or_exit(arg, "missing expression").as_str());
        input.push_str(" ");
        arg = args.next();
        if arg.is_none() {
            break;
        }
    }

    // Attempt to parse the input expression.
    let root = parse(input.as_str());
    let root = ok_or_exit(root);

    // Echo the parsed expression.
    let mut stdout = stdout();
    let mut writer = match lang.as_deref() {
        Some("dot") => graph::GraphWriter::new_dot(&mut stdout),
        Some("mermaid") => graph::GraphWriter::new_mermaid(&mut stdout),
        _ => unreachable!(),
    };

    ok_or_exit(writer.write(root.as_ref()));
}

fn print_roll(roll: &DieRoll) {
    let DieRoll {
        sides,
        result,
        keep,
    } = roll;

    if *keep {
        print!("{KEEP}[{KEEP:#}{KEEP_DIE}d{sides}{KEEP_DIE:#}{KEEP}:{KEEP:#}{KEEP_VAL}{result}{KEEP_VAL:#}{KEEP}]{KEEP:#}")
    } else {
        print!("{DROP}[{DROP:#}{DROP_DIE}d{sides}{DROP_DIE:#}{DROP}:{DROP:#}{DROP_VAL}{result}{DROP_VAL:#}{DROP}]{DROP:#}")
    }
}

fn main() {
    // The expression to evaluate is given on the command line and may be
    // preceded by 'min', 'mid', or 'max' to specify the evaluation strategy,
    // or by 'dot' or 'mermaid' to output the syntax tree as a graph. The
    // remaining arguments are concatenated to form a single expression.
    let mut args = env::args().map(|arg| arg.to_lowercase());
    args.next();
    let arg = args.next();

    if let Some("dot" | "mermaid") = arg.as_deref() {
        graph(arg, &mut args);
    } else {
        eval(arg, &mut args);
    }
}
