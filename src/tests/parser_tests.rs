// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

use crate::{
    ast::{Node, Selection},
    parser::{parse, Error},
};

#[test]
fn test_literals() {
    assert_eq!(parse("5"), Ok(Box::new(Node::Lit { value: 5 })));
    assert_eq!(parse("42"), Ok(Box::new(Node::Lit { value: 42 })));
}

#[test]
fn test_sums() {
    assert_eq!(
        parse("1 + 2"),
        Ok(Box::new(Node::Add {
            left: Box::new(Node::Lit { value: 1 }),
            right: Box::new(Node::Lit { value: 2 }),
        }))
    );
    assert_eq!(
        parse("3-4"),
        Ok(Box::new(Node::Sub {
            left: Box::new(Node::Lit { value: 3 }),
            right: Box::new(Node::Lit { value: 4 }),
        }))
    );
    assert_eq!(
        parse("1 + 2 - 3 + 4"),
        Ok(Box::new(Node::Add {
            left: Box::new(Node::Sub {
                left: Box::new(Node::Add {
                    left: Box::new(Node::Lit { value: 1 }),
                    right: Box::new(Node::Lit { value: 2 }),
                }),
                right: Box::new(Node::Lit { value: 3 }),
            }),
            right: Box::new(Node::Lit { value: 4 }),
        }))
    );
    assert!(matches!(parse("1 + 2 - "), Err(Error::UnexpectedEnd(_))));
    assert!(matches!(parse("1 - + "), Err(Error::UnexpectedToken(_))));
}

#[test]
fn test_terms() {
    assert_eq!(
        parse("1 * 2"),
        Ok(Box::new(Node::Mul {
            left: Box::new(Node::Lit { value: 1 }),
            right: Box::new(Node::Lit { value: 2 }),
        }))
    );
    assert_eq!(
        parse("3/4"),
        Ok(Box::new(Node::Div {
            left: Box::new(Node::Lit { value: 3 }),
            right: Box::new(Node::Lit { value: 4 }),
        }))
    );
    assert_eq!(
        parse("1 * 2 / 3 * 4"),
        Ok(Box::new(Node::Mul {
            left: Box::new(Node::Div {
                left: Box::new(Node::Mul {
                    left: Box::new(Node::Lit { value: 1 }),
                    right: Box::new(Node::Lit { value: 2 }),
                }),
                right: Box::new(Node::Lit { value: 3 }),
            }),
            right: Box::new(Node::Lit { value: 4 }),
        }))
    );
    assert!(matches!(parse("1 * 2 / "), Err(Error::UnexpectedEnd(_))));
    assert!(matches!(parse("1 * / "), Err(Error::UnexpectedToken(_))));
}

#[test]
fn test_negation() {
    assert_eq!(
        parse("-5"),
        Ok(Box::new(Node::Neg {
            right: Box::new(Node::Lit { value: 5 }),
        }))
    );
    assert_eq!(
        parse("1--5"),
        Ok(Box::new(Node::Sub {
            left: Box::new(Node::Lit { value: 1 }),
            right: Box::new(Node::Neg {
                right: Box::new(Node::Lit { value: 5 }),
            }),
        }))
    );
    assert_eq!(
        parse("-1 + -2"),
        Ok(Box::new(Node::Add {
            left: Box::new(Node::Neg {
                right: Box::new(Node::Lit { value: 1 }),
            }),
            right: Box::new(Node::Neg {
                right: Box::new(Node::Lit { value: 2 }),
            }),
        }))
    );
    assert!(matches!(parse("-"), Err(Error::UnexpectedEnd(_))));
}

#[test]
fn test_dice() {
    assert_eq!(
        parse("d4"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 4 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("d6"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("d8"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 8 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("d10"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 10 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("d12"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 12 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("d20"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 20 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("d100"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 100 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("d%"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 100 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("d"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("2d8"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 2 }),
            sides: Box::new(Node::Lit { value: 8 }),
            select: None,
        }))
    );
    assert_eq!(
        parse("3d"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 3 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: None,
        }))
    );
    assert!(matches!(parse("d7"), Err(Error::InvalidDie(_))));
}

#[test]
fn test_selection() {
    assert_eq!(
        parse("d20adv"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 20 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::Advantage,
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("d20ad"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 20 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::Advantage,
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("d20dis"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 20 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::Disadvantage,
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("d20da"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 1 }),
            sides: Box::new(Node::Lit { value: 20 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::Disadvantage,
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("4d6kh3"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 4 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::KeepHighest {
                    count: Some(Box::new(Node::Lit { value: 3 }))
                },
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("4d k3"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 4 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::KeepHighest {
                    count: Some(Box::new(Node::Lit { value: 3 }))
                },
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("4d6kl3"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 4 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::KeepLowest {
                    count: Some(Box::new(Node::Lit { value: 3 }))
                },
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("4d6dl1"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 4 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::DropLowest {
                    count: Some(Box::new(Node::Lit { value: 1 }))
                },
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("4d d1"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 4 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::DropLowest {
                    count: Some(Box::new(Node::Lit { value: 1 }))
                },
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("4d6dh1"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 4 }),
            sides: Box::new(Node::Lit { value: 6 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::DropHighest {
                    count: Some(Box::new(Node::Lit { value: 1 }))
                },
                next: None
            })),
        }))
    );
    assert_eq!(
        parse("3d8 d1 adv k1"),
        Ok(Box::new(Node::Roll {
            count: Box::new(Node::Lit { value: 3 }),
            sides: Box::new(Node::Lit { value: 8 }),
            select: Some(Box::new(Node::Select {
                selection: Selection::DropLowest {
                    count: Some(Box::new(Node::Lit { value: 1 }))
                },
                next: Some(Box::new(Node::Select {
                    selection: Selection::Advantage,
                    next: Some(Box::new(Node::Select {
                        selection: Selection::KeepHighest {
                            count: Some(Box::new(Node::Lit { value: 1 }))
                        },
                        next: None
                    })),
                }))
            })),
        }))
    );
}

#[test]
fn test_op_precedence() {
    assert_eq!(
        parse("1 + 2 * 3"),
        Ok(Box::new(Node::Add {
            left: Box::new(Node::Lit { value: 1 }),
            right: Box::new(Node::Mul {
                left: Box::new(Node::Lit { value: 2 }),
                right: Box::new(Node::Lit { value: 3 }),
            }),
        }))
    );
    assert_eq!(
        parse("4 / 2 - 1"),
        Ok(Box::new(Node::Sub {
            left: Box::new(Node::Div {
                left: Box::new(Node::Lit { value: 4 }),
                right: Box::new(Node::Lit { value: 2 }),
            }),
            right: Box::new(Node::Lit { value: 1 }),
        }))
    );
    assert_eq!(
        parse("1 + 2 * -3 - 4 / -5"),
        Ok(Box::new(Node::Sub {
            left: Box::new(Node::Add {
                left: Box::new(Node::Lit { value: 1 }),
                right: Box::new(Node::Mul {
                    left: Box::new(Node::Lit { value: 2 }),
                    right: Box::new(Node::Neg {
                        right: Box::new(Node::Lit { value: 3 }),
                    }),
                }),
            }),
            right: Box::new(Node::Div {
                left: Box::new(Node::Lit { value: 4 }),
                right: Box::new(Node::Neg {
                    right: Box::new(Node::Lit { value: 5 }),
                }),
            }),
        }))
    );
    assert_eq!(
        parse("(1 + -2) * [3 - 4] / -5"),
        Ok(Box::new(Node::Div {
            left: Box::new(Node::Mul {
                left: Box::new(Node::Add {
                    left: Box::new(Node::Lit { value: 1 }),
                    right: Box::new(Node::Neg {
                        right: Box::new(Node::Lit { value: 2 }),
                    })
                }),
                right: Box::new(Node::Sub {
                    left: Box::new(Node::Lit { value: 3 }),
                    right: Box::new(Node::Lit { value: 4 }),
                }),
            }),
            right: Box::new(Node::Neg {
                right: Box::new(Node::Lit { value: 5 }),
            }),
        }))
    );
    assert!(matches!(
        parse("(1 + 2] * [3 - 4) / 2"),
        Err(Error::MismatchedParentheses(_))
    ));
}

#[test]
fn test_roll_precedence() {
    assert_eq!(
        parse("1 + d6 * 2"),
        Ok(Box::new(Node::Add {
            left: Box::new(Node::Lit { value: 1 }),
            right: Box::new(Node::Mul {
                left: Box::new(Node::Roll {
                    count: Box::new(Node::Lit { value: 1 }),
                    sides: Box::new(Node::Lit { value: 6 }),
                    select: None,
                }),
                right: Box::new(Node::Lit { value: 2 }),
            }),
        }))
    );
    assert_eq!(
        parse("d20adv + d20dis"),
        Ok(Box::new(Node::Add {
            left: Box::new(Node::Roll {
                count: Box::new(Node::Lit { value: 1 }),
                sides: Box::new(Node::Lit { value: 20 }),
                select: Some(Box::new(Node::Select {
                    selection: Selection::Advantage,
                    next: None
                })),
            }),
            right: Box::new(Node::Roll {
                count: Box::new(Node::Lit { value: 1 }),
                sides: Box::new(Node::Lit { value: 20 }),
                select: Some(Box::new(Node::Select {
                    selection: Selection::Disadvantage,
                    next: None
                })),
            }),
        }))
    );
}
