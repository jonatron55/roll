// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! Abstract Syntax Tree (AST) for dice expressions.
//!
//! This module defines the nodes that make up the AST for dice expressions as
//! well as a `Visitor` trait that can be used to traverse the AST.

use std::error::Error;

/// An abstract node in the syntax tree.
pub trait Node: std::fmt::Debug {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult;
}

pub type VisitorResult = Result<(), Box<dyn Error>>;

/// A visitor trait that can be implemented to traverse the AST.
pub trait Visitor {
    /// Visit a literal node.
    fn lit(&mut self, node: &Lit) -> VisitorResult;

    /// Visit a roll node.
    fn roll(&mut self, node: &Roll) -> VisitorResult;

    /// Visit a select node.
    fn select(&mut self, node: &Select) -> VisitorResult;

    /// Visit a negate node.
    fn neg(&mut self, node: &Neg) -> VisitorResult;

    /// Visit an add node.
    fn add(&mut self, node: &Add) -> VisitorResult;

    /// Visit a subtract node.
    fn sub(&mut self, node: &Sub) -> VisitorResult;

    /// Visit a multiply node.
    fn mul(&mut self, node: &Mul) -> VisitorResult;

    /// Visit a divide node.
    fn div(&mut self, node: &Div) -> VisitorResult;
}

/// Selections that can be made over dice rolls.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Selection {
    /// Keep the lowest *n* dice.
    KeepLowest,

    /// Keep the highest *n* dice.
    KeepHighest,

    /// Discard the lowest *n* dice.
    DropLowest,

    /// Discard the highest *n* dice.
    DropHighest,

    /// Reroll the previous expression and keep the higher total.
    Advantage,

    /// Reroll the previous expression and keep the lower total.
    Disadvantage,
}

/// Node that represents a literal integer value.
#[derive(Debug)]
pub struct Lit {
    pub value: i32,
}

/// A node that represents rolling some number of particular dice.
#[derive(Debug)]
pub struct Roll {
    pub count: Box<dyn Node>,
    pub sides: Box<dyn Node>,
    pub select: Option<Box<dyn Node>>,
}

/// A node that specifies some selection over previously rolled dice.
#[derive(Debug)]
pub struct Select {
    pub selection: Selection,
    pub count: Option<Box<dyn Node>>,
    pub next: Option<Box<dyn Node>>,
}

/// A node that represents the unary negation operation.
#[derive(Debug)]
pub struct Neg {
    pub right: Box<dyn Node>,
}

/// A node that represents the addition operation.
#[derive(Debug)]
pub struct Add {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

/// A node that represents the subtraction operation.
#[derive(Debug)]
pub struct Sub {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

/// A node that represents the multiplication operation.
#[derive(Debug)]
pub struct Mul {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

/// A node that represents the division operation.
#[derive(Debug)]
pub struct Div {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

impl Node for Lit {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.lit(self)
    }
}

impl Node for Roll {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.roll(self)
    }
}

impl Node for Select {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.select(self)
    }
}

impl Node for Neg {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.neg(self)
    }
}

impl Node for Add {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.add(self)
    }
}

impl Node for Sub {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.sub(self)
    }
}

impl Node for Mul {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.mul(self)
    }
}

impl Node for Div {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.div(self)
    }
}
