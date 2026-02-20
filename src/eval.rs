// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! This module contains an evaluator for dice expressions that traverses an
//! AST and returns the result of the expression.

use std::{
    cmp::Ordering,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Range,
};

use rand::{Rng, RngExt};

use crate::ast::{Node, Selection};

/// Possible ways to evaluate dice rolls.
pub enum Evaluation<TRng: Rng> {
    /// Randomly generate each die roll.
    Rand(TRng),

    /// Evaluate the expression as if all dice rolls were 1.
    Min,

    /// Evaluate the expression as if all dice rolls landed on the median of
    /// their range (which will be fractional for even-sided dice).
    Mid,

    /// Evaluate the expression as if all dice rolls were the highest possible.
    Max,

    /// Evaluate the expression using fixed die rolls. Panics if there are not
    /// enough rolls provided.
    #[cfg(test)]
    Mocked(Vec<i32>),
}

/// A single die roll.
#[derive(Debug, PartialEq)]
pub struct DieRoll {
    /// The number of sides on the die. May be 4, 6, 8, 10, 12, 20, or 100.
    pub sides: i32,

    /// The result of the roll.
    pub result: f64,

    /// Whether the roll was kept or discarded during a selection operation.
    pub keep: bool,
}

/// Evaluates each node in the AST using a stack and returns the result of the
/// expression along with the individual die rolls.
pub struct Evaluator<TRng: Rng> {
    /// The rolls made during the evaluation.
    pub rolls: Vec<DieRoll>,

    /// The evaluation strategy to use.
    evaluation: Evaluation<TRng>,

    /// A stack of intermediate results. Once the traversal is complete, the
    /// stack should contain a single value representing the result of the
    /// expression.
    results: Vec<f64>,

    /// A stack of dice pools that are being selected from. Pools are ranges
    /// over the `rolls` vector and are pushed when a new roll is made and
    /// then modified by selection operations.
    dice_pools: Vec<Range<usize>>,
}

/// Possible errors that can occur during evaluation.
#[derive(Debug, PartialEq)]
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

    pub fn eval(&mut self, node: &Node) -> Result<i32, Error> {
        self.rolls.clear();
        self.visit(node)?;
        self.results
            .pop()
            .ok_or(Error::StackUnderflow)
            .map(|x| x.floor() as i32)
    }

    fn visit(&mut self, node: &Node) -> Result<(), Error> {
        match node {
            Node::Lit { value } => self.lit(*value),
            Node::Roll {
                count,
                sides,
                select,
            } => self.roll(&count, &sides, select.as_ref().map(|n| n.as_ref())),
            Node::Select { selection, next } => {
                self.select(selection, next.as_ref().map(|n| n.as_ref()))
            }
            Node::Neg { right } => self.neg(right.as_ref()),
            Node::Add { left, right } => self.add(left.as_ref(), right.as_ref()),
            Node::Sub { left, right } => self.sub(left.as_ref(), right.as_ref()),
            Node::Mul { left, right } => self.mul(left.as_ref(), right.as_ref()),
            Node::Div { left, right } => self.div(left.as_ref(), right.as_ref()),
        }
    }

    fn lit(&mut self, value: i32) -> Result<(), Error> {
        self.results.push(value as f64);
        Ok(())
    }

    fn roll(&mut self, count: &Node, sides: &Node, select: Option<&Node>) -> Result<(), Error> {
        self.visit(count)?;
        let Some(count) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };
        let count = count as usize;

        self.visit(sides)?;
        let Some(sides) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };
        let sides = sides as i32;

        let roll_start = self.rolls.len();

        for _ in 0..count {
            let roll = match &mut self.evaluation {
                Evaluation::Rand(rng) => rng.random_range(1..sides + 1) as f64,
                Evaluation::Min => 1.0,
                Evaluation::Mid => sides as f64 * 0.5 + 0.5,
                Evaluation::Max => sides as f64,
                #[cfg(test)]
                Evaluation::Mocked(rolls) => rolls.remove(0) as f64,
            };

            self.rolls.push(DieRoll {
                sides,
                result: roll,
                keep: true,
            });
        }

        let pool = roll_start..self.rolls.len();
        if let Some(select) = &select {
            self.dice_pools.push(pool.clone());
            self.visit(select)?;
            self.dice_pools.pop();
        }

        self.rolls[pool.start..pool.end]
            .sort_unstable_by(|a, b| b.result.partial_cmp(&a.result).unwrap_or(Ordering::Equal));

        let total = self.rolls[roll_start..self.rolls.len()]
            .iter()
            .map(|r| if r.keep { r.result } else { 0.0 })
            .sum();

        self.results.push(total);

        Ok(())
    }

    fn select(&mut self, selection: &Selection, next: Option<&Node>) -> Result<(), Error> {
        let pool = match self.dice_pools.last() {
            Some(pool) => pool.clone(),
            None => return Err(Error::StackUnderflow),
        };

        match selection {
            Selection::KeepHighest { count }
            | Selection::DropHighest { count }
            | Selection::KeepLowest { count }
            | Selection::DropLowest { count } => {
                // Sort the pool appropriately and select the dice to keep/drop
                let high = matches!(selection, Selection::KeepHighest { .. })
                    || matches!(selection, Selection::DropHighest { .. });
                let keep = matches!(selection, Selection::KeepHighest { .. })
                    || matches!(selection, Selection::KeepLowest { .. });

                let count = match &count {
                    Some(child) => {
                        self.visit(child)?;
                        self.results.pop().ok_or(Error::StackUnderflow)? as usize
                    }
                    None => 1,
                };

                if count > pool.len() {
                    return Err(Error::InvalidSelection {
                        selection_size: count,
                        pool_size: pool.len(),
                    });
                }

                if high {
                    self.rolls[pool.start..pool.end].sort_unstable_by(|a, b| {
                        b.result.partial_cmp(&a.result).unwrap_or(Ordering::Equal)
                    });
                } else {
                    self.rolls[pool.start..pool.end].sort_unstable_by(|a, b| {
                        a.result.partial_cmp(&b.result).unwrap_or(Ordering::Equal)
                    });
                }

                for i in 0..count {
                    self.rolls[pool.start + i].keep = keep;
                }

                for i in count..pool.len() {
                    self.rolls[pool.start + i].keep = !keep;
                }

                if let Some(next) = &next {
                    let remaining = if keep {
                        pool.start..pool.start + count
                    } else {
                        pool.start + count..pool.end
                    };

                    self.dice_pools.push(remaining);
                    self.visit(next)?;
                    self.dice_pools.pop();
                }

                Ok(())
            }

            Selection::Advantage | Selection::Disadvantage => {
                // Reroll the current pool and select the highest/lowest total of the two rolls
                for i in pool.clone() {
                    let sides = self.rolls[i].sides;
                    let roll = match &mut self.evaluation {
                        Evaluation::Rand(rng) => rng.random_range(1..sides + 1) as f64,
                        Evaluation::Min => 1.0,
                        Evaluation::Mid => sides as f64 * 0.5 + 0.5,
                        Evaluation::Max => sides as f64,
                        #[cfg(test)]
                        Evaluation::Mocked(rolls) => rolls.remove(0) as f64,
                    };

                    self.rolls.push(DieRoll {
                        sides,
                        result: roll,
                        keep: true,
                    });
                }

                let old = pool.start..pool.end;
                let new = self.rolls.len() - pool.len()..self.rolls.len();

                let total_old: f64 = self.rolls[old.clone()].iter().map(|r| r.result).sum();
                let total_new: f64 = self.rolls[new.clone()].iter().map(|r| r.result).sum();
                let kept = if (total_new > total_old) == matches!(selection, Selection::Advantage) {
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

                if let Some(next) = &next {
                    self.dice_pools.push(kept);
                    self.visit(next)?;
                    self.dice_pools.pop();
                }

                Ok(())
            }
        }
    }

    fn neg(&mut self, right: &Node) -> Result<(), Error> {
        self.visit(right)?;
        let Some(right) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };
        self.results.push(-right);
        Ok(())
    }

    fn add(&mut self, left: &Node, right: &Node) -> Result<(), Error> {
        self.visit(left)?;
        let Some(left) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };
        self.visit(right)?;
        let Some(right) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };

        self.results.push(left + right);
        Ok(())
    }

    fn sub(&mut self, left: &Node, right: &Node) -> Result<(), Error> {
        self.visit(left)?;
        let Some(left) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };
        self.visit(right)?;
        let Some(right) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };

        self.results.push(left - right);
        Ok(())
    }

    fn mul(&mut self, left: &Node, right: &Node) -> Result<(), Error> {
        self.visit(left)?;
        let Some(left) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };
        self.visit(right)?;
        let Some(right) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };

        self.results.push(left * right);
        Ok(())
    }

    fn div(&mut self, left: &Node, right: &Node) -> Result<(), Error> {
        self.visit(left)?;
        let Some(left) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };
        self.visit(right)?;
        let Some(right) = self.results.pop() else {
            return Err(Error::StackUnderflow);
        };

        if right.abs() < f64::EPSILON {
            return Err(Error::DivideByZero);
        }

        self.results.push(left / right);
        Ok(())
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
