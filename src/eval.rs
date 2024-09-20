// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! This module contains an evaluator for dice expressions that traverses an
//! AST and returns the result of the expression.

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Range,
};

use rand::Rng;

use crate::ast::{
    Add, Div, Lit, Mul, Neg, Node, Roll, Select, Selection, Sub, Visitor, VisitorResult,
};

/// Possible ways to evaluate dice rolls.
pub enum Evaluation<TRng: Rng> {
    /// Randomly generate each die roll.
    Rand(TRng),

    /// Evaluate the expression as if all dice rolls were 1.
    Min,

    /// Evaluate the expression as if all dice rolls landed in the middle of
    /// their range (rounded down).
    Mid,

    /// Evaluate the expression as if all dice rolls were the highest possible.
    Max,
}

/// A single die roll.
pub struct DieRoll {
    /// The number of sides on the die. May be 4, 6, 8, 10, 12, 20, or 100.
    pub sides: i32,

    /// The result of the roll.
    pub result: i32,

    /// Whether the roll was kept or discarded during a selection operation.
    pub keep: bool,
}

/// An implementation of the `Visitor` trait that evaluates each node in the AST
/// using a stack and returns the result of the expression along with the
/// individual die rolls.
pub struct Evaluator<TRng: Rng> {
    /// The rolls made during the evaluation.
    pub rolls: Vec<DieRoll>,

    /// The evaluation strategy to use.
    evaluation: Evaluation<TRng>,

    /// A stack of intermediate results. Once the traversal is complete, the
    /// stack should contain a single value representing the result of the
    /// expression.
    results: Vec<i32>,

    /// A stack of dice pools that are being selected from. Pools are ranges
    /// over the `rolls` vector and are pushed when a new roll is made and
    /// then modified by selection operations.
    dice_pools: Vec<Range<usize>>,
}

/// Possible errors that can occur during evaluation.
#[derive(Debug)]
pub enum Error {
    /// A selection operation (`kh`, `dl`, etc.) involves selecting more dice
    /// than are available after rolling and applying previous selections.
    InvalidSelection {
        selection_size: usize,
        pool_size: usize,
    },

    /// An attempt was made to divide by zero.
    DivideByZero,

    /// The stack was empty when an operation expected a value.
    StackUnderflow,
}

impl<TRng: Rng> Evaluator<TRng> {
    pub fn new(evaluation: Evaluation<TRng>) -> Self {
        Self {
            evaluation,
            rolls: Vec::new(),
            results: Vec::new(),
            dice_pools: Vec::new(),
        }
    }

    pub fn eval(&mut self, node: &dyn Node) -> Result<i32, Box<dyn std::error::Error>> {
        self.rolls.clear();
        node.accept(self)?;

        self.results.pop().ok_or(Box::new(Error::StackUnderflow))
    }
}

impl<TRng: Rng> Visitor for Evaluator<TRng> {
    fn lit(&mut self, node: &Lit) -> VisitorResult {
        self.results.push(node.value);
        Ok(())
    }

    fn roll(&mut self, node: &Roll) -> VisitorResult {
        node.count.accept(self)?;
        let Some(count) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };

        node.sides.accept(self)?;
        let Some(sides) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };

        for _ in 0..count {
            let roll = match &mut self.evaluation {
                Evaluation::Rand(rng) => rng.gen_range(1..sides + 1),
                Evaluation::Min => 1,
                Evaluation::Mid => sides / 2,
                Evaluation::Max => sides,
            };

            self.rolls.push(DieRoll {
                sides,
                result: roll,
                keep: true,
            });
        }

        let pool = self.rolls.len() - count as usize..self.rolls.len();
        if let Some(select) = &node.select {
            self.dice_pools.push(pool.clone());
            select.accept(self)?;
            self.dice_pools.pop();
        }

        self.rolls[pool.start..pool.end].sort_unstable_by(|a, b| b.result.cmp(&a.result));

        let total = self
            .rolls
            .iter()
            .map(|r| if r.keep { r.result } else { 0 })
            .sum();

        self.results.push(total);

        Ok(())
    }

    fn select(&mut self, node: &Select) -> VisitorResult {
        let pool = match self.dice_pools.last() {
            Some(pool) => pool.clone(),
            None => return Err(Box::new(Error::StackUnderflow)),
        };

        match node.selection {
            Selection::KeepHighest
            | Selection::DropHighest
            | Selection::KeepLowest
            | Selection::DropLowest => {
                // Sort the pool appropriately and select the dice to keep/drop
                let high = node.selection == Selection::KeepHighest
                    || node.selection == Selection::DropHighest;
                let keep = node.selection == Selection::KeepHighest
                    || node.selection == Selection::KeepLowest;

                let count = match &node.count {
                    Some(child) => {
                        child.accept(self)?;
                        self.results.pop().ok_or(Box::new(Error::StackUnderflow))? as usize
                    }
                    None => 1,
                };

                if count > pool.len() {
                    return Err(Box::new(Error::InvalidSelection {
                        selection_size: count,
                        pool_size: pool.len(),
                    }));
                }

                if high {
                    self.rolls[pool.start..pool.end]
                        .sort_unstable_by(|a, b| b.result.cmp(&a.result));
                } else {
                    self.rolls[pool.start..pool.end]
                        .sort_unstable_by(|a, b| a.result.cmp(&b.result));
                }

                for i in 0..count {
                    self.rolls[pool.start + i].keep = keep;
                }

                for i in count..pool.len() {
                    self.rolls[pool.start + i].keep = !keep;
                }

                if let Some(next) = &node.next {
                    let remaining = if keep {
                        pool.start..pool.start + count
                    } else {
                        pool.start + count..pool.end
                    };

                    self.dice_pools.push(remaining);
                    next.accept(self)?;
                    self.dice_pools.pop();
                }

                Ok(())
            }

            Selection::Advantage | Selection::Disadvantage => {
                // Reroll the current pool and select the highest/lowest total of the two rolls
                for i in pool.clone() {
                    let sides = self.rolls[i].sides;
                    let roll = match &mut self.evaluation {
                        Evaluation::Rand(rng) => rng.gen_range(1..sides + 1),
                        Evaluation::Min => 1,
                        Evaluation::Mid => sides / 2,
                        Evaluation::Max => sides,
                    };

                    self.rolls.push(DieRoll {
                        sides,
                        result: roll,
                        keep: true,
                    });
                }

                let old = pool.start..pool.end;
                let new = self.rolls.len() - pool.len()..self.rolls.len();

                let total_old: i32 = self.rolls[old.clone()].iter().map(|r| r.result).sum();
                let total_new: i32 = self.rolls[new.clone()].iter().map(|r| r.result).sum();
                let kept = if (total_new > total_old) == (node.selection == Selection::Advantage) {
                    for roll in old {
                        self.rolls[roll].keep = false
                    }
                    new
                } else {
                    for roll in new {
                        self.rolls[roll].keep = false
                    }
                    old
                };

                if let Some(next) = &node.next {
                    self.dice_pools.push(kept);
                    next.accept(self)?;
                    self.dice_pools.pop();
                }

                Ok(())
            }
        }
    }

    fn neg(&mut self, node: &Neg) -> VisitorResult {
        node.right.accept(self)?;
        let Some(right) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };
        self.results.push(-right);
        Ok(())
    }

    fn add(&mut self, node: &Add) -> VisitorResult {
        node.left.accept(self)?;
        let Some(left) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };
        node.right.accept(self)?;
        let Some(right) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };

        self.results.push(left + right);
        Ok(())
    }

    fn sub(&mut self, node: &Sub) -> VisitorResult {
        node.left.accept(self)?;
        let Some(left) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };
        node.right.accept(self)?;
        let Some(right) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };

        self.results.push(left - right);
        Ok(())
    }

    fn mul(&mut self, node: &Mul) -> VisitorResult {
        node.left.accept(self)?;
        let Some(left) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };
        node.right.accept(self)?;
        let Some(right) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };

        self.results.push(left * right);
        Ok(())
    }

    fn div(&mut self, node: &Div) -> VisitorResult {
        node.left.accept(self)?;
        let Some(left) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };
        node.right.accept(self)?;
        let Some(right) = self.results.pop() else {
            return Err(Box::new(Error::StackUnderflow));
        };

        if right == 0 {
            return Err(Box::new(Error::DivideByZero));
        }

        self.results.push(left / right);
        Ok(())
    }
}

impl Display for DieRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.keep {
            write!(
                f,
                "\x1B[32m[d{}:\x1B[22m\x1B[1m{}\x1B[22m]\x1B[39m",
                self.sides, self.result
            )
        } else {
            write!(
                f,
                "\x1B[9m\x1B[31m[d{}:\x1B[22m\x1B[1m{}\x1B[22m]\x1B[39m\x1B[29m",
                self.sides, self.result
            )
        }
    }
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::InvalidSelection {
                selection_size,
                pool_size,
            } => write!(
                f,
                "Cannot select {} dice from a pool of {}",
                selection_size, pool_size
            ),
            Error::DivideByZero => write!(f, "Division by zero"),
            Error::StackUnderflow => write!(f, "Stack underflow"),
        }
    }
}
