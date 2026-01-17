// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

use crate::lexer::{Error, Lexer, Token};

/// Test that each type of valid token is recognized correctly.
#[test]
fn test_tokens() {
    let input = "+-*/%()[]×÷ d k kh kl dh dl adv dis da ad 1 42";
    let mut lexer = Lexer::new(input);

    let expected_tokens = vec![
        Token::Plus,
        Token::Minus,
        Token::Times,
        Token::Divide,
        Token::Percent,
        Token::Open('('),
        Token::Close(')'),
        Token::Open('['),
        Token::Close(']'),
        Token::Times,
        Token::Divide,
        Token::Word("d"),
        Token::Word("k"),
        Token::Word("kh"),
        Token::Word("kl"),
        Token::Word("dh"),
        Token::Word("dl"),
        Token::Word("adv"),
        Token::Word("dis"),
        Token::Word("da"),
        Token::Word("ad"),
        Token::Integer(1),
        Token::Integer(42),
    ];

    for expected in expected_tokens {
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, expected);
    }

    assert!(lexer.next().is_none());
}

/// Test that tokens are recognized when whitespace is absent and when extraneous whitespace is present.
#[test]
fn test_whitespace() {
    let input = "  + \n - \t *12da/% ";
    let mut lexer = Lexer::new(input);

    let expected_tokens = vec![
        Token::Plus,
        Token::Minus,
        Token::Times,
        Token::Integer(12),
        Token::Word("da"),
        Token::Divide,
        Token::Percent,
    ];

    for expected in expected_tokens {
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, expected);
    }

    assert!(lexer.next().is_none());
}

/// Test that invalid characters produce an error.
#[test]
fn test_invalid_character() {
    let input = "+ - $";
    let mut lexer = Lexer::new(input);

    assert_eq!(lexer.next().unwrap().unwrap(), Token::Plus);
    assert_eq!(lexer.next().unwrap().unwrap(), Token::Minus);
    let err = lexer.next().unwrap().unwrap_err();
    assert_eq!(err, Error::InvalidCharacter('$'));
}

/// Test that invalid words produce an error.
#[test]
fn test_invalid_word() {
    let input = "d splort kh";
    let mut lexer = Lexer::new(input);

    assert_eq!(lexer.next().unwrap().unwrap(), Token::Word("d"));
    let err = lexer.next().unwrap().unwrap_err();
    assert_eq!(err, Error::InvalidWord("splort".to_string()));
}

/// Test that invalid integers produce an error.
#[test]
fn test_invalid_integer() {
    let input = "d 9223372036854775808 kh";
    let mut lexer = Lexer::new(input);

    assert_eq!(lexer.next().unwrap().unwrap(), Token::Word("d"));
    let err = lexer.next().unwrap().unwrap_err();
    assert!(matches!(err, Error::ParseIntError(_)));
}
