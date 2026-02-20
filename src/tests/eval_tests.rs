// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

use rand::rngs::ThreadRng;

use crate::{
    eval::{DieRoll, Error, Evaluation, Evaluator},
    parser::parse,
};

#[test]
fn test_arithmetic() {
    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Max);
    assert_eq!(
        evaluator.eval(parse("2 + 3 * 4 - 6 / 2").unwrap().as_ref()),
        Ok(11)
    );
    assert_eq!(
        evaluator.eval(parse("(2 + 3) * (4 - 6) / 2").unwrap().as_ref()),
        Ok(-5)
    );
}

#[test]
fn test_rolls() {
    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![10, 2, 4, 6, 12, 6]));

    let result = evaluator.eval(parse("d20 + 5").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![DieRoll {
            sides: 20,
            result: 10,
            keep: true
        }]
    );
    assert_eq!(result, Ok(15));

    let result = evaluator.eval(parse("3d6").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 6,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 4,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: true
            }
        ]
    );
    assert_eq!(result, Ok(12));

    let result = evaluator.eval(parse("d20 + d6").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 20,
                result: 12,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 6,
                keep: true
            },
        ]
    );
    assert_eq!(result, Ok(18));
}

#[test]
fn test_selection() {
    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![15, 5]));
    let result = evaluator.eval(parse("d20ad").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 20,
                result: 15,
                keep: true
            },
            DieRoll {
                sides: 20,
                result: 5,
                keep: false
            },
        ]
    );
    assert_eq!(result, Ok(15));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![15, 5]));
    let result = evaluator.eval(parse("d20da").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 20,
                result: 15,
                keep: false
            },
            DieRoll {
                sides: 20,
                result: 5,
                keep: true
            },
        ]
    );
    assert_eq!(result, Ok(5));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![2, 3, 5, 6]));
    let result = evaluator.eval(parse("4d6kh3").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 6,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 5,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 3,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: false
            },
        ]
    );
    assert_eq!(result, Ok(14));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![2, 3, 5, 6]));
    let result = evaluator.eval(parse("4d6dl1").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 6,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 5,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 3,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: false
            },
        ]
    );
    assert_eq!(result, Ok(14));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![2, 3, 5, 6]));
    let result = evaluator.eval(parse("4d6kl3").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 6,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 5,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 3,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: true
            },
        ]
    );
    assert_eq!(result, Ok(10));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![2, 3, 5, 6]));
    let result = evaluator.eval(parse("4d6dh1").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 6,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 5,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 3,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: true
            },
        ]
    );
    assert_eq!(result, Ok(10));
}

#[test]
fn test_chained_selection() {
    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![1, 4, 6, 2]));
    let result = evaluator.eval(parse("4d6kh3d1").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 6,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 4,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 1,
                keep: false
            },
        ]
    );
    assert_eq!(result, Ok(10));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![2, 3, 5, 6, 1, 1, 1]));
    let result = evaluator.eval(parse("4d6kh3ad").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 6,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 5,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 3,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 1,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 1,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 1,
                keep: false
            },
        ]
    );
    assert_eq!(result, Ok(14));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![2, 3, 5, 6, 4, 4, 4]));
    let result = evaluator.eval(parse("4d6dl1da").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 6,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 5,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 3,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 4,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 4,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 4,
                keep: true
            },
        ]
    );
    assert_eq!(result, Ok(12));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![1, 2, 3, 4, 6, 5, 5]));
    let result = evaluator.eval(parse("4d6dh1ad").unwrap().as_ref());
    assert_eq!(
        evaluator.rolls,
        vec![
            DieRoll {
                sides: 6,
                result: 4,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 3,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 2,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 1,
                keep: false
            },
            DieRoll {
                sides: 6,
                result: 6,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 5,
                keep: true
            },
            DieRoll {
                sides: 6,
                result: 5,
                keep: true
            },
        ]
    );
    assert_eq!(result, Ok(16));
}

#[test]
fn test_invalid_selection() {
    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![1, 2, 3, 4]));
    let result = evaluator.eval(parse("4d6kh5").unwrap().as_ref());
    assert!(matches!(result, Err(Error::InvalidSelection { .. })));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![1, 2, 3, 4]));
    let result = evaluator.eval(parse("4d6kh2dl3").unwrap().as_ref());
    assert!(matches!(result, Err(Error::InvalidSelection { .. })));
}

#[test]
fn test_div0() {
    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![1, 2, 3, 4]));
    let result = evaluator.eval(parse("4d6 / 0").unwrap().as_ref());
    assert!(matches!(result, Err(Error::DivideByZero)));

    let mut evaluator = Evaluator::new(Evaluation::<ThreadRng>::Mocked(vec![1, 2, 3, 4]));
    let result = evaluator.eval(parse("3d6 / (d6 - 4)").unwrap().as_ref());
    assert!(matches!(result, Err(Error::DivideByZero)));
}
