// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! A recursive descent parser for dice expressions. See `README.md` for a
//! formal grammar of the language.

use std::fmt::Display;

use crate::ast::{Add, Div, Lit, Mul, Neg, Node, Roll, Select, Selection, Sub};
use crate::lexer::{Lexer, Token};
use crate::lookahead::Lookahead;

type LookaheadLexer<'a> = Lookahead<Lexer<'a>>;

/// Parsing errors.
#[derive(Debug)]
pub enum Error {
    /// A token was encountered at an unexpected position.
    UnexpectedToken(String),

    /// The end of the input was reached unexpectedly.
    UnexpectedEnd(String),

    /// A die with an invalid number of sides was encountered.
    InvalidDie(String),

    /// A closing parenthesis was encountered that did not match an opening
    /// parenthesis.
    MismatchedParentheses(String),
}

type Result = std::result::Result<Box<dyn Node>, Error>;
type ResultOption = std::result::Result<Option<Box<dyn Node>>, Error>;

/// Parse a dice expression into an abstract syntax tree.
pub fn parse<'a>(input: &'a str) -> Result {
    let lexer = Lexer::new(input);
    let mut lexer = Lookahead::new(lexer);
    let root = parse_root(&mut lexer);

    if lexer.peek().is_some() {
        Err(Error::UnexpectedToken(format!(
            "Unexpected leftover token: {:?}",
            lexer.peek()
        )))
    } else {
        root
    }
}

/// Parse the production rule:
/// ```ebnf
/// root = sum;
/// ```
fn parse_root(lexer: &mut LookaheadLexer) -> Result {
    parse_sum(lexer)
}

/// Parse the production rule:
/// ```ebnf
/// sum = term, { ("+" | "-"), term };
/// ```
fn parse_sum(lexer: &mut LookaheadLexer) -> Result {
    let mut left = parse_term(lexer)?;

    loop {
        match lexer.peek() {
            Some(Token::Plus) => {
                lexer.next();
                let right = parse_term(lexer)?;
                left = Box::new(Add { left, right });
            }
            Some(Token::Minus) => {
                lexer.next();
                let right = parse_term(lexer)?;
                left = Box::new(Sub { left, right });
            }
            _ => break,
        }
    }

    Ok(left)
}
/// Parse the production rule:
/// ```ebnf
/// term = factor, { ("*" | "/"), factor };
/// ```
fn parse_term(lexer: &mut LookaheadLexer) -> Result {
    let mut left = parse_factor(lexer)?;

    loop {
        match lexer.peek() {
            Some(Token::Times) => {
                lexer.next();
                let right = parse_factor(lexer)?;
                left = Box::new(Mul { left, right });
            }
            Some(Token::Divide) => {
                lexer.next();
                let right = parse_factor(lexer)?;
                left = Box::new(Div { left, right });
            }
            _ => break,
        }
    }

    Ok(left)
}

/// Parse the production rule:
/// ```ebnf
/// factor = "(", sum, ")" | negation | integer | roll;
/// ```
fn parse_factor(lexer: &mut LookaheadLexer) -> Result {
    let token = lexer.peek().cloned();

    match token {
        Some(Token::Open(open_ch)) => {
            lexer.next();
            let sum = parse_sum(lexer)?;

            match lexer.peek().cloned() {
                Some(Token::Close(close_ch)) => {
                    lexer.next();
                    match (open_ch, close_ch) {
                        ('(', ')') => Ok(sum),
                        ('[', ']') => Ok(sum),
                        _ => Err(Error::MismatchedParentheses(format!(
                            "Closing '{close_ch}' does not match opening '{open_ch}'"
                        ))),
                    }
                }
                Some(other) => Err(Error::UnexpectedToken(format!(
                    "{:?} unexpected in parenthetical",
                    other
                ))),
                None => Err(Error::UnexpectedEnd(format!(
                    "Expression ended without closing '{}'",
                    match open_ch {
                        '(' => ')',
                        '[' => ']',
                        _ => unreachable!(),
                    }
                ))),
            }
        }

        Some(Token::Integer(n)) => {
            let token = lexer.next();

            match &token {
                Some(Token::Word("d")) => parse_roll(lexer, n),
                _ => Ok(Box::new(Lit { value: n })),
            }
        }

        Some(Token::Word("d")) => parse_roll(lexer, 1),

        Some(Token::Minus) => {
            lexer.next();
            let right = parse_factor(lexer)?;
            Ok(Box::new(Neg { right }))
        }

        Some(other) => Err(Error::UnexpectedToken(format!(
            "{:?} unexpected in factor",
            other
        ))),

        None => Err(Error::UnexpectedEnd("Unexpected end of input".to_string())),
    }
}

/// Parse the production rule:
/// ```ebnf
/// roll = [integer], "d", [integer], [selection];
/// ```
fn parse_roll(lexer: &mut LookaheadLexer, count: i32) -> Result {
    let token = lexer.peek();
    match token {
        Some(Token::Word("d")) => {
            let token = lexer.next();

            match token {
                Some(Token::Integer(sides)) => match sides {
                    4 | 6 | 8 | 10 | 12 | 20 | 100 => {
                        lexer.next();
                        let select = parse_selection(lexer)?;
                        Ok(Box::new(Roll {
                            count: Box::new(Lit { value: count }),
                            sides: Box::new(Lit { value: sides }),
                            select,
                        }))
                    }
                    _ => Err(Error::InvalidDie(format!("Invalid die: d{sides}"))),
                },
                Some(Token::Percent) => {
                    lexer.next();
                    let select = parse_selection(lexer)?;
                    Ok(Box::new(Roll {
                        count: Box::new(Lit { value: count }),
                        sides: Box::new(Lit { value: 100 }),
                        select,
                    }))
                }
                _ => {
                    let select = parse_selection(lexer)?;
                    Ok(Box::new(Roll {
                        count: Box::new(Lit { value: count }),
                        sides: Box::new(Lit { value: 6 }),
                        select,
                    }))
                }
            }
        }
        _ => Err(Error::UnexpectedToken(format!(
            "{:?} unexpected in roll",
            token
        ))),
    }
}

/// Parse the production rule:
/// ```ebnf
/// selection = (
///         "k", integer |
///         "kh", integer |
///         "kl", integer |
///         "d", integer |
///         "dh", integer |
///         "dl", integer |
///         "adv" | "ad" |
///         "dis" | "da"
///     ), [selection];
/// ```
fn parse_selection(lexer: &mut LookaheadLexer) -> ResultOption {
    let token = lexer.peek();
    match token {
        Some(Token::Word("k"))
        | Some(Token::Word("kh"))
        | Some(Token::Word("kl"))
        | Some(Token::Word("d"))
        | Some(Token::Word("dh"))
        | Some(Token::Word("dl")) => {
            let selection = match token {
                Some(Token::Word("k")) => Selection::KeepHighest,
                Some(Token::Word("kh")) => Selection::KeepHighest,
                Some(Token::Word("kl")) => Selection::KeepLowest,
                Some(Token::Word("d")) => Selection::DropLowest,
                Some(Token::Word("dh")) => Selection::DropHighest,
                Some(Token::Word("dl")) => Selection::DropLowest,
                _ => unreachable!(),
            };

            let token = lexer.next();
            match token {
                Some(Token::Integer(n)) => {
                    lexer.next();
                    Ok(Some(Box::new(Select {
                        selection,
                        count: Some(Box::new(Lit { value: n })),
                        next: parse_selection(lexer)?,
                    })))
                }

                _ => Ok(Some(Box::new(Select {
                    selection,
                    count: None,
                    next: parse_selection(lexer)?,
                }))),
            }
        }

        Some(Token::Word("adv")) | Some(Token::Word("ad")) => {
            lexer.next();
            Ok(Some(Box::new(Select {
                selection: Selection::Advantage,
                count: None,
                next: parse_selection(lexer)?,
            })))
        }

        Some(Token::Word("dis")) | Some(Token::Word("da")) => {
            lexer.next();
            Ok(Some(Box::new(Select {
                selection: Selection::Disadvantage,
                count: None,
                next: parse_selection(lexer)?,
            })))
        }
        _ => Ok(None),
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::UnexpectedToken(message) => write!(f, "{message}",),
            Error::UnexpectedEnd(message) => write!(f, "{message}"),
            Error::InvalidDie(message) => write!(f, "{message}"),
            Error::MismatchedParentheses(message) => write!(f, "{message}",),
        }
    }
}
