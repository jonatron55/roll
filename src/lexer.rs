// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! Lexical analyzer for dice expressions.
//!
//! Dice expressions are broken into tokens according to the following rules:
//!
//! - Whitespace is discarded as it is encountered and is only significant when
//!   separating words or integers.
//! - Contiguous sequences of decimal digits as tokenized as integers.
//! - Contiguous sequences of alphabetic characters are tokenized as words. The
//!   following words are recognized as valid: `d`, `k`, `kh`, `kl`, `dh`, `dl`,
//!  `adv`, `dis`, `da`, `ad`.
//! - Words not listed above must not appear in the expression.
//! - The following symbols are recognized as distinct tokens: `+`, `-`, `*`,
//!   `/`, `%`, `(`, `)`, `[`, `]`. The symbols `×` and `÷` are also recognized
//!   as equivalent to `*` and `/`, respectively.
//! - No other characters may appear in the expression.

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    num::ParseIntError,
    str::CharIndices,
};

const VALID_WORDS: &'static [&'static str] =
    &["d", "k", "kh", "kl", "dh", "dl", "adv", "dis", "da", "ad"];

/// The types of tokens that can be produced by the lexer.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    /// An integer literal.
    Integer(i32),

    /// A recognized word.
    Word(&'a str),

    /// The symbol `+`.
    Plus,

    /// The symbol `-`.
    Minus,

    /// The symbol `*` or `×`.
    Times,

    /// The symbol `/` or `÷`.
    Divide,

    /// The symbol `%`.
    Percent,

    /// The symbol `(` or `[`.
    Open(char),

    /// The symbol `)` or `]`.
    Close(char),
}

/// A lexical analyzer for dice expressions. The lexer implements an `Iterator`
/// over tokens in the input expression.
pub struct Lexer<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    current: Option<(usize, char)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    InvalidCharacter(char),
    InvalidWord(String),
    ParseIntError(ParseIntError),
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let chars = input.char_indices();

        Lexer {
            input,
            chars,
            current: None,
        }
    }

    fn peek(&self) -> Option<char> {
        self.current.map(|(_, c)| c)
    }

    fn peek_position(&self) -> usize {
        self.current.map_or(self.input.len(), |(i, _)| i)
    }

    fn next_char(&mut self) -> Option<char> {
        self.current = self.chars.next();
        self.peek()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Result<Token<'a>, Error>> {
        if self.current.is_none() {
            self.current = self.chars.next();
        }

        // Consume whitespace
        while self.peek().map_or(false, |c| c.is_whitespace()) {
            self.next_char();
        }

        let Some(ch) = self.peek() else {
            // End of input
            return None;
        };

        if ch.is_digit(10) {
            // Consume an integer (take all contiguous digits)
            let i = self.peek_position();

            while self.next_char().map_or(false, |c| c.is_digit(10)) {}

            let j = self.peek_position();
            let n = match self.input[i..j].parse() {
                Ok(n) => n,
                Err(err) => return Some(Err(Error::ParseIntError(err))),
            };
            return Some(Ok(Token::Integer(n)));
        }

        if ch.is_alphabetic() {
            // Consume a word (take all contiguous alphabetic characters)
            let i = self.peek_position();

            while self.next_char().map_or(false, |c| c.is_alphabetic()) {}

            let j = self.peek_position();
            let word = &self.input[i..j];

            if !VALID_WORDS.contains(&word) {
                return Some(Err(Error::InvalidWord(word.to_string())));
            }

            return Some(Ok(Token::Word(word)));
        }

        // Otherwise, consume a single-character symbol
        self.next_char();
        match ch {
            '+' => Some(Ok(Token::Plus)),
            '-' => Some(Ok(Token::Minus)),
            '*' | '×' => Some(Ok(Token::Times)),
            '/' | '÷' => Some(Ok(Token::Divide)),
            '%' => Some(Ok(Token::Percent)),
            '(' => Some(Ok(Token::Open('('))),
            '[' => Some(Ok(Token::Open('['))),
            ')' => Some(Ok(Token::Close(')'))),
            ']' => Some(Ok(Token::Close(']'))),
            _ => Some(Err(Error::InvalidCharacter(ch))),
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Token::Integer(i) => write!(f, "{}", i),
            Token::Word(word) => write!(f, "{}", word),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Times => write!(f, "×"),
            Token::Divide => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Open(ch) => write!(f, "{ch}"),
            Token::Close(ch) => write!(f, "{ch}"),
        }
    }
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::InvalidCharacter(ch) => write!(f, "Invalid character '{}'", ch),
            Error::InvalidWord(word) => write!(f, "Invalid word '{}'", word),
            Error::ParseIntError(err) => write!(f, "{}", err),
        }
    }
}
