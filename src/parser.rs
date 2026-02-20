// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! A recursive descent parser for dice expressions. See `README.md` for a
//! formal grammar of the language.

use std::{error::Error as StdError, fmt::Display, fmt::Formatter, fmt::Result as FmtResult};

use crate::ast::{Node, Selection};
use crate::lexer::{Error as LexError, Lexer, Token};
use crate::lookahead::Lookahead;

type LookaheadLexer<'a> = Lookahead<Lexer<'a>>;

/// Parsing errors.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// A token was encountered at an unexpected position.
    UnexpectedToken(String),

    /// The end of the input was reached unexpectedly.
    UnexpectedEnd(String),

    /// A die with an invalid number of sides was encountered.
    InvalidDie(String),

    /// A closing `)` or `]` was encountered that did not match the opening `(`
    /// or `[`.
    MismatchedParentheses(String),

    /// An error occurred in the lexer.
    LexError(LexError),
}

type Result = std::result::Result<Box<Node>, Error>;
type ResultOption = std::result::Result<Option<Box<Node>>, Error>;

/// Parse a dice expression into an abstract syntax tree.
pub fn parse(input: &str) -> Result {
    let lexer = Lexer::new(input);
    let mut lexer = Lookahead::new(lexer);
    let root = parse_root(&mut lexer)?;

    match lexer.peek() {
        Some(Ok(token)) => Err(Error::UnexpectedToken(format!(
            "Unexpected leftover token: '{token}'"
        ))),
        Some(Err(err)) => Err(err.into()),
        _ => Ok(root),
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
            Some(Ok(Token::Plus)) => {
                lexer.next();
                let right = parse_term(lexer)?;
                left = Box::new(Node::Add { left, right });
            }
            Some(Ok(Token::Minus)) => {
                lexer.next();
                let right = parse_term(lexer)?;
                left = Box::new(Node::Sub { left, right });
            }
            Some(Err(err)) => return Err(err.into()),
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
            Some(Ok(Token::Times)) => {
                lexer.next();
                let right = parse_factor(lexer)?;
                left = Box::new(Node::Mul { left, right });
            }
            Some(Ok(Token::Divide)) => {
                lexer.next();
                let right = parse_factor(lexer)?;
                left = Box::new(Node::Div { left, right });
            }
            Some(Err(err)) => return Err(err.into()),
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
        Some(Ok(Token::Open(open_ch))) => {
            lexer.next();
            let sum = parse_sum(lexer)?;

            match lexer.peek().cloned() {
                Some(Ok(Token::Close(close_ch))) => {
                    lexer.next();
                    match (open_ch, close_ch) {
                        ('(', ')') => Ok(sum),
                        ('[', ']') => Ok(sum),
                        _ => Err(Error::MismatchedParentheses(format!(
                            "Closing '{close_ch}' does not match opening '{open_ch}'"
                        ))),
                    }
                }
                Some(Ok(other)) => Err(Error::UnexpectedToken(format!(
                    "'{other}' unexpected in parenthetical",
                ))),
                Some(Err(err)) => return Err(err.into()),
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

        Some(Ok(Token::Integer(n))) => {
            let token = lexer.next();

            match &token {
                Some(Ok(Token::Word("d"))) => parse_roll(lexer, n),
                _ => Ok(Box::new(Node::Lit { value: n })),
            }
        }

        Some(Ok(Token::Word("d"))) => parse_roll(lexer, 1),

        Some(Ok(Token::Minus)) => {
            lexer.next();
            let right = parse_factor(lexer)?;
            Ok(Box::new(Node::Neg { right }))
        }

        Some(Err(err)) => return Err(err.into()),

        Some(Ok(other)) => Err(Error::UnexpectedToken(format!(
            "'{other}' unexpected in factor",
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
        Some(Ok(Token::Word("d"))) => {
            let token = lexer.next();

            match token {
                Some(Ok(Token::Integer(sides))) => match sides {
                    4 | 6 | 8 | 10 | 12 | 20 | 100 => {
                        lexer.next();
                        let select = parse_selection(lexer)?;
                        Ok(Box::new(Node::Roll {
                            count: Box::new(Node::Lit { value: count }),
                            sides: Box::new(Node::Lit { value: sides }),
                            select,
                        }))
                    }
                    _ => Err(Error::InvalidDie(format!("Invalid die: d{sides}"))),
                },
                Some(Ok(Token::Percent)) => {
                    lexer.next();
                    let select = parse_selection(lexer)?;
                    Ok(Box::new(Node::Roll {
                        count: Box::new(Node::Lit { value: count }),
                        sides: Box::new(Node::Lit { value: 100 }),
                        select,
                    }))
                }

                Some(Err(err)) => return Err(err.into()),

                _ => {
                    let select = parse_selection(lexer)?;
                    Ok(Box::new(Node::Roll {
                        count: Box::new(Node::Lit { value: count }),
                        sides: Box::new(Node::Lit { value: 6 }),
                        select,
                    }))
                }
            }
        }

        Some(Err(err)) => return Err(err.into()),

        Some(Ok(other)) => Err(Error::UnexpectedToken(format!(
            "'{other}' unexpected in roll",
        ))),

        None => Err(Error::UnexpectedEnd("Unexpected end of input".to_string())),
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
        Some(Ok(Token::Word("k")))
        | Some(Ok(Token::Word("kh")))
        | Some(Ok(Token::Word("kl")))
        | Some(Ok(Token::Word("d")))
        | Some(Ok(Token::Word("dh")))
        | Some(Ok(Token::Word("dl"))) => {
            let selection = *token.unwrap().as_ref().unwrap();
            let token = lexer.next();
            let (count, next) = match token {
                Some(Ok(Token::Integer(n))) => {
                    lexer.next();
                    (
                        Some(Box::new(Node::Lit { value: n })),
                        parse_selection(lexer)?,
                    )
                }

                Some(Err(err)) => return Err(err.into()),

                _ => (None, parse_selection(lexer)?),
            };

            match selection {
                Token::Word("k") | Token::Word("kh") => Ok(Some(Box::new(Node::Select {
                    selection: Selection::KeepHighest { count },
                    next,
                }))),
                Token::Word("kl") => Ok(Some(Box::new(Node::Select {
                    selection: Selection::KeepLowest { count },
                    next,
                }))),
                Token::Word("d") | Token::Word("dl") => Ok(Some(Box::new(Node::Select {
                    selection: Selection::DropLowest { count },
                    next,
                }))),
                Token::Word("dh") => Ok(Some(Box::new(Node::Select {
                    selection: Selection::DropHighest { count },
                    next,
                }))),
                _ => unreachable!(),
            }
        }

        Some(Ok(Token::Word("adv"))) | Some(Ok(Token::Word("ad"))) => {
            lexer.next();
            Ok(Some(Box::new(Node::Select {
                selection: Selection::Advantage,
                next: parse_selection(lexer)?,
            })))
        }

        Some(Ok(Token::Word("dis"))) | Some(Ok(Token::Word("da"))) => {
            lexer.next();
            Ok(Some(Box::new(Node::Select {
                selection: Selection::Disadvantage,
                next: parse_selection(lexer)?,
            })))
        }

        Some(Err(err)) => return Err(err.into()),

        _ => Ok(None),
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::LexError(error) => Some(error),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::UnexpectedToken(message) => write!(f, "{message}",),
            Error::UnexpectedEnd(message) => write!(f, "{message}"),
            Error::InvalidDie(message) => write!(f, "{message}"),
            Error::MismatchedParentheses(message) => write!(f, "{message}",),
            Error::LexError(error) => write!(f, "{error}"),
        }
    }
}

impl From<LexError> for Error {
    fn from(error: LexError) -> Self {
        Error::LexError(error)
    }
}

impl From<&LexError> for Error {
    fn from(error: &LexError) -> Self {
        Error::LexError(error.clone())
    }
}
