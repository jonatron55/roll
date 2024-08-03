// Recursive descent parser for dice expressions matching the following grammar:
//
// root -> sum
// sum -> term [('+' | '-') term]
// term -> product | sum
// product -> factor [('*' | '/') factor]
// factor -> '(' sum ')' | negation |integer | roll | product
// negation -> '-' factor
// roll -> [integer] 'd' [integer] [selection]
// selection -> ('k' integer | 'kh' integer | 'kl' integer | 'dh' integer | 'dl' integer | 'adv' | 'dis' | 'da' | 'ad') [selection]

use std::fmt::Display;

use crate::ast::{Add, Divide, Literal, Multiply, Negate, Node, Roll, Select, Selection, Subtract};
use crate::lexer::{Lexer, Token};
use crate::lookahead::Lookahead;

type LookaheadLexer<'a> = Lookahead<Lexer<'a>>;

#[derive(Debug)]
pub enum Error {
    UnexpectedToken(String),
    UnexpectedEnd(String),
    InvalidDie(String),
    MismatchedParentheses(String),
}

type Result = std::result::Result<Box<dyn Node>, Error>;
type ResultOption = std::result::Result<Option<Box<dyn Node>>, Error>;

pub fn parse<'a>(input: &'a str) -> Result {
    let lexer = Lexer::new(input);
    let mut lexer = Lookahead::new(lexer);
    let root = parse_root(&mut lexer);

    if lexer.peek().is_some() {
        Err(Error::UnexpectedToken(format!(
            "Unexpected token: {:?}",
            lexer.peek()
        )))
    } else {
        root
    }
}

fn parse_root(lexer: &mut LookaheadLexer) -> Result {
    parse_sum(lexer)
}

fn parse_sum(lexer: &mut LookaheadLexer) -> Result {
    let term = parse_term(lexer)?;

    match lexer.peek() {
        Some(Token::Plus) => {
            lexer.next();
            let right: Box<dyn Node> = parse_term(lexer)?;
            Ok(Box::new(Add { left: term, right }))
        }

        Some(Token::Minus) => {
            lexer.next();
            let right = parse_term(lexer)?;
            Ok(Box::new(Subtract { left: term, right }))
        }

        _ => Ok(term),
    }
}

fn parse_term(lexer: &mut LookaheadLexer) -> Result {
    let product = parse_product(lexer)?;

    match lexer.peek() {
        Some(Token::Plus) => {
            lexer.next();
            let right = parse_product(lexer)?;
            Ok(Box::new(Add {
                left: product,
                right,
            }))
        }

        Some(Token::Minus) => {
            lexer.next();
            let right = parse_product(lexer)?;
            Ok(Box::new(Subtract {
                left: product,
                right,
            }))
        }

        _ => Ok(product),
    }
}

fn parse_product(lexer: &mut LookaheadLexer) -> Result {
    let factor = parse_factor(lexer)?;

    match lexer.peek() {
        Some(Token::Times) => {
            lexer.next();
            let right = parse_factor(lexer)?;
            Ok(Box::new(Multiply {
                left: factor,
                right,
            }))
        }

        Some(Token::Divide) => {
            lexer.next();
            let right = parse_factor(lexer)?;
            Ok(Box::new(Divide {
                left: factor,
                right,
            }))
        }

        _ => Ok(factor),
    }
}

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
                Some(Token::Times) | Some(Token::Divide) => {
                    lexer.next();
                    let right = parse_factor(lexer)?;
                    Ok(Box::new(Multiply {
                        left: Box::new(Literal { value: n }),
                        right,
                    }))
                }
                _ => Ok(Box::new(Literal { value: n })),
            }
        }

        Some(Token::Word("d")) => parse_roll(lexer, 1),

        Some(Token::Minus) => {
            lexer.next();
            let right = parse_factor(lexer)?;
            Ok(Box::new(Negate { right }))
        }

        Some(other) => Err(Error::UnexpectedToken(format!(
            "{:?} unexpected in factor",
            other
        ))),

        None => Err(Error::UnexpectedEnd("Unexpected end of input".to_string())),
    }
}

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
                            count: Box::new(Literal { value: count }),
                            sides: Box::new(Literal { value: sides }),
                            select,
                        }))
                    }
                    _ => Err(Error::InvalidDie(format!("Invalid die: d{sides}"))),
                },
                Some(Token::Percent) => {
                    lexer.next();
                    let select = parse_selection(lexer)?;
                    Ok(Box::new(Roll {
                        count: Box::new(Literal { value: count }),
                        sides: Box::new(Literal { value: 100 }),
                        select,
                    }))
                }
                _ => {
                    let select = parse_selection(lexer)?;
                    Ok(Box::new(Roll {
                        count: Box::new(Literal { value: count }),
                        sides: Box::new(Literal { value: 6 }),
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
                        count: Some(Box::new(Literal { value: n })),
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
