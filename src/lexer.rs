use core::panic;
use std::str::CharIndices;

const VALID_WORDS: &'static [&'static str] = &["d", "k", "kh", "kl", "dh", "dl", "adv", "dis", "da", "ad"];

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    Integer(i32),
    Word(&'a str),
    Plus,
    Minus,
    Times,
    Divide,
    Percent,
    Open(char),
    Close(char),
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    current: Option<(usize, char)>,
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

    fn next(&mut self) -> Option<char> {
        self.current = self.chars.next();
        self.peek()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        if self.current.is_none() {
            self.current = self.chars.next();
        }

        // Consume whitespace
        while self.peek().map_or(false, |c| c.is_whitespace()) {
            self.next();
        }

        let Some(ch) = self.peek() else {
            return None;
        };

        if ch.is_digit(10) {
            let i = self.peek_position();

            while self.next().map_or(false, |c| c.is_digit(10)) {}

            let j = self.peek_position();

            return Some(Token::Integer(self.input[i..j].parse().unwrap()));
        }

        if ch.is_alphabetic() {
            let i = self.peek_position();

            while self.next().map_or(false, |c| c.is_alphabetic()) {}

            let j = self.peek_position();
            let word = &self.input[i..j];

            if !VALID_WORDS.contains(&word) {
                panic!("Invalid word: \"{}\"", word);
            }

            return Some(Token::Word(word));
        }

        self.next();
        match ch {
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' | '×' => Some(Token::Times),
            '/' | '÷' => Some(Token::Divide),
            '%' => Some(Token::Percent),
            '(' => Some(Token::Open('(')),
            '[' => Some(Token::Open('[')),
            ')' => Some(Token::Close(')')),
            ']' => Some(Token::Close(']')),
            _ => None,
        }
    }
}
