// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! Abstract Syntax Tree (AST) for dice expressions.
//!
//! This module defines the nodes that make up the AST for dice expressions.

/// Selections that can be made over dice rolls.
#[derive(Debug)]
pub enum Selection {
    /// Keep the lowest *n* dice.
    KeepLowest { count: Option<Box<Node>> },

    /// Keep the highest *n* dice.
    KeepHighest { count: Option<Box<Node>> },

    /// Discard the lowest *n* dice.
    DropLowest { count: Option<Box<Node>> },

    /// Discard the highest *n* dice.
    DropHighest { count: Option<Box<Node>> },

    /// Reroll the previous expression and keep the higher total.
    Advantage,

    /// Reroll the previous expression and keep the lower total.
    Disadvantage,
}

/// A node in the syntax tree.
#[derive(Debug)]
pub enum Node {
    /// Node that represents a literal integer value.
    Lit { value: i32 },

    /// A node that represents rolling some number of particular dice.
    Roll {
        count: Box<Node>,
        sides: Box<Node>,
        select: Option<Box<Node>>,
    },

    /// A node that specifies some selection over previously rolled dice.
    Select {
        selection: Selection,
        next: Option<Box<Node>>,
    },

    /// A node that represents the unary negation operation.
    Neg { right: Box<Node> },

    /// A node that represents the addition operation.
    Add { left: Box<Node>, right: Box<Node> },

    /// A node that represents the subtraction operation.
    Sub { left: Box<Node>, right: Box<Node> },

    /// A node that represents the multiplication operation.
    Mul { left: Box<Node>, right: Box<Node> },

    /// A node that represents the division operation.
    Div { left: Box<Node>, right: Box<Node> },
}
