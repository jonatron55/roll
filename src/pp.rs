// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! This module contains a pretty-printer for the dice expressions that
//! traverses an AST and outputs a string representation of the expression.

use std::io::Write;

use crate::ast::{Add, Div, Lit, Mul, Neg, Roll, Select, Selection, Sub, Visitor, VisitorResult};

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
}

impl<'o, W: Write> Visitor for PP<'o, W> {
    fn lit(&mut self, node: &Lit) -> VisitorResult {
        write!(self.writer, "{}", node.value)?;
        Ok(())
    }

    fn roll(&mut self, node: &Roll) -> VisitorResult {
        node.count.accept(self)?;
        write!(self.writer, "d")?;
        node.sides.accept(self)?;

        if let Some(selection) = &node.select {
            selection.accept(self)?;
        }

        Ok(())
    }

    fn select(&mut self, node: &Select) -> VisitorResult {
        match node.selection {
            Selection::KeepHighest => write!(self.writer, "kh")?,
            Selection::KeepLowest => write!(self.writer, "kl")?,
            Selection::DropHighest => write!(self.writer, "dh")?,
            Selection::DropLowest => write!(self.writer, "dl")?,
            Selection::Advantage => write!(self.writer, "adv")?,
            Selection::Disadvantage => write!(self.writer, "dis")?,
        };

        if let Some(count) = &node.count {
            count.accept(self)?;
        }

        Ok(())
    }

    fn neg(&mut self, node: &Neg) -> VisitorResult {
        let was_prod = self.prod;
        self.prod = true;
        write!(self.writer, "-")?;
        node.right.accept(self)?;
        self.prod = was_prod;
        Ok(())
    }

    fn add(&mut self, node: &Add) -> VisitorResult {
        let was_prod = self.prod;
        self.prod = false;

        if was_prod {
            write!(self.writer, "(")?;
        }

        node.left.accept(self)?;
        write!(self.writer, " + ")?;
        node.right.accept(self)?;

        if was_prod {
            write!(self.writer, ")")?;
        }

        self.prod = was_prod;
        Ok(())
    }

    fn sub(&mut self, node: &Sub) -> VisitorResult {
        let was_prod = self.prod;
        self.prod = false;

        if was_prod {
            write!(self.writer, "(")?;
        }

        node.left.accept(self)?;
        write!(self.writer, " - ")?;
        node.right.accept(self)?;

        if was_prod {
            write!(self.writer, ")")?;
        }

        self.prod = was_prod;
        Ok(())
    }

    fn mul(&mut self, node: &Mul) -> VisitorResult {
        let was_prod = self.prod;
        self.prod = true;

        node.left.accept(self)?;
        write!(self.writer, " × ")?;
        node.right.accept(self)?;

        self.prod = was_prod;
        Ok(())
    }

    fn div(&mut self, node: &Div) -> VisitorResult {
        let was_prod = self.prod;
        self.prod = true;

        node.left.accept(self)?;
        write!(self.writer, " / ")?;
        node.right.accept(self)?;

        self.prod = was_prod;
        Ok(())
    }
}
