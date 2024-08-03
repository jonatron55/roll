use std::{fmt::Display, ops::Range};

use rand::Rng;

use crate::ast::{
    Add, Divide, Literal, Multiply, Negate, Node, Roll, Select, Selection, Subtract, Visitor,
    VisitorResult,
};

pub enum Evaluation<TRng: Rng> {
    Random(TRng),
    Min,
    Mid,
    Max,
}

#[derive(Debug)]
pub struct DieRoll {
    pub sides: i32,
    pub result: i32,
    pub keep: bool,
}

pub struct Evaluator<TRng: Rng> {
    pub rolls: Vec<DieRoll>,
    evaluation: Evaluation<TRng>,
    results: Vec<i32>,
    dice_pools: Vec<Range<usize>>,
}

#[derive(Debug)]
pub enum Error {
    InvalidSelection {
        selection_size: usize,
        pool_size: usize,
    },
    DivideByZero,
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
    fn literal(&mut self, node: &Literal) -> VisitorResult {
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
                Evaluation::Random(rng) => rng.gen_range(1..sides + 1),
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
        let pool = self.dice_pools.last().unwrap().clone();

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
                        self.results.pop().unwrap() as usize
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
                        Evaluation::Random(rng) => rng.gen_range(1..sides + 1),
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

    fn negate(&mut self, node: &Negate) -> VisitorResult {
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

    fn subtract(&mut self, node: &Subtract) -> VisitorResult {
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

    fn multiply(&mut self, node: &Multiply) -> VisitorResult {
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

    fn divide(&mut self, node: &Divide) -> VisitorResult {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.keep {
            write!(f, "\x1B[32m[d{}:\x1B[22m\x1B[1m{}\x1B[22m]\x1B[39m", self.sides, self.result)
        } else {
            write!(f, "\x1B[9m\x1B[31m[d{}:\x1B[22m\x1B[1m{}\x1B[22m]\x1B[39m\x1B[29m", self.sides, self.result)
        }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidSelection{ selection_size, pool_size } => write!(f, "Cannot select {} dice from a pool of {}", selection_size, pool_size),
            Error::DivideByZero => write!(f, "Division by zero"),
            Error::StackUnderflow => write!(f, "Stack underflow"),
        }
    }
}
