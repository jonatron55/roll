// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! This module contains a pretty-printer for the dice expressions that
//! traverses an AST and outputs a string representation of the expression.

use std::fmt::{Error as FmtError, Write};

use crate::ast::{Node, Selection};

/// A pretty-printer for dice expressions.
pub struct PP<'o, W: Write> {
    /// The writer to which the pretty-printed expression is written.
    writer: &'o mut W,

    /// Whether the most recent operation node is a product (in which case,
    /// terms will require parentheses).
    prod: bool,
}

impl<'o, W: Write> PP<'o, W> {
    pub fn new(writer: &'o mut W) -> Self {
        Self {
            writer,
            prod: false,
        }
    }

    pub fn write(&mut self, node: &Node) -> Result<(), FmtError> {
        self.visit(node)
    }

    fn visit(&mut self, node: &Node) -> Result<(), FmtError> {
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

    fn lit(&mut self, value: i32) -> Result<(), FmtError> {
        write!(self.writer, "{}", value)?;
        Ok(())
    }

    fn roll(&mut self, count: &Node, sides: &Node, select: Option<&Node>) -> Result<(), FmtError> {
        self.visit(count)?;
        write!(self.writer, "d")?;
        self.visit(sides)?;

        if let Some(selection) = &select {
            self.visit(selection)?;
        }

        Ok(())
    }

    fn select(&mut self, selection: &Selection, next: Option<&Node>) -> Result<(), FmtError> {
        match selection {
            Selection::KeepHighest { .. } => write!(self.writer, "kh")?,
            Selection::KeepLowest { .. } => write!(self.writer, "kl")?,
            Selection::DropHighest { .. } => write!(self.writer, "dh")?,
            Selection::DropLowest { .. } => write!(self.writer, "dl")?,
            Selection::Advantage => write!(self.writer, "adv")?,
            Selection::Disadvantage => write!(self.writer, "dis")?,
        };

        match selection {
            Selection::KeepHighest { count }
            | Selection::KeepLowest { count }
            | Selection::DropHighest { count }
            | Selection::DropLowest { count } => {
                if let Some(count) = count {
                    self.visit(&count)?
                } else {
                    write!(self.writer, "1")?;
                }
            }
            _ => {}
        };

        if let Some(next) = &next {
            self.visit(next)?;
        }

        Ok(())
    }

    fn neg(&mut self, right: &Node) -> Result<(), FmtError> {
        let was_prod = self.prod;
        self.prod = true;
        write!(self.writer, "-")?;
        self.visit(right)?;
        self.prod = was_prod;
        Ok(())
    }

    fn add(&mut self, left: &Node, right: &Node) -> Result<(), FmtError> {
        let was_prod = self.prod;
        self.prod = false;

        if was_prod {
            write!(self.writer, "(")?;
        }

        self.visit(left)?;
        write!(self.writer, " + ")?;
        self.visit(right)?;

        if was_prod {
            write!(self.writer, ")")?;
        }

        self.prod = was_prod;
        Ok(())
    }

    fn sub(&mut self, left: &Node, right: &Node) -> Result<(), FmtError> {
        let was_prod = self.prod;
        self.prod = false;

        if was_prod {
            write!(self.writer, "(")?;
        }

        self.visit(left)?;
        write!(self.writer, " - ")?;
        self.visit(right)?;

        if was_prod {
            write!(self.writer, ")")?;
        }

        self.prod = was_prod;
        Ok(())
    }

    fn mul(&mut self, left: &Node, right: &Node) -> Result<(), FmtError> {
        let was_prod = self.prod;
        self.prod = true;

        self.visit(left)?;
        write!(self.writer, " × ")?;
        self.visit(right)?;

        self.prod = was_prod;
        Ok(())
    }

    fn div(&mut self, left: &Node, right: &Node) -> Result<(), FmtError> {
        let was_prod = self.prod;
        self.prod = true;

        self.visit(left)?;
        write!(self.writer, " / ")?;
        self.visit(right)?;

        self.prod = was_prod;
        Ok(())
    }
}
