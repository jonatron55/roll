// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! This module outputs a Graphviz DOT file representing the AST of a dice expression.

use std::{
    io::{Error as IoError, Write},
    result::Result,
};

use crate::ast::{
    Add, Div, Lit, Mul, Neg, Node, Roll, Select, Selection, Sub, Visitor, VisitorResult,
};

/// A pretty-printer for dice expressions.
pub struct DotWriter<'o, W: Write> {
    /// The writer to which the pretty-printed expression is written.
    writer: &'o mut W,

    /// The unique identifier for the next node to be written.
    next_id: usize,

    /// A stack of node identifiers to track the current path through the AST.
    id_stack: Vec<String>,
}

impl<'o, W: Write> DotWriter<'o, W> {
    pub fn new(writer: &'o mut W) -> Self {
        Self {
            writer,
            next_id: 1,
            id_stack: Vec::new(),
        }
    }

    pub fn write(&mut self, root: &dyn Node) -> VisitorResult {
        writeln!(self.writer, "graph {{")?;
        writeln!(self.writer, "    node [shape=\"rect\"]")?;
        writeln!(self.writer, "    edge [fontsize=\"10\"]")?;
        root.accept(self)?;
        writeln!(self.writer, "}}")?;
        Ok(())
    }

    fn write_node(&mut self, label: &str) -> Result<String, IoError> {
        let id = format!("node{:04x}", self.next_id);
        self.next_id += 1;
        writeln!(self.writer, "    {} [label=\"{}\"]", id, label)?;
        Ok(id)
    }

    fn write_edge(&mut self, parent: &str, child: &str, label: &str) -> Result<(), IoError> {
        writeln!(
            self.writer,
            "    {} -- {} [label=\"{}\"]",
            parent, child, label
        )?;
        Ok(())
    }
}

impl<'o, W: Write> Visitor for DotWriter<'o, W> {
    fn lit(&mut self, node: &Lit) -> VisitorResult {
        let id = self.write_node(&format!("{}", node.value))?;
        self.id_stack.push(id);
        Ok(())
    }

    fn roll(&mut self, node: &Roll) -> VisitorResult {
        let id = self.write_node("Roll")?;
        node.count.accept(self)?;
        node.sides.accept(self)?;

        let count_id = self.id_stack.pop().unwrap();
        let sides_id = self.id_stack.pop().unwrap();

        self.write_edge(&id, &count_id, "count")?;
        self.write_edge(&id, &sides_id, "sides")?;

        if let Some(selection) = &node.select {
            selection.accept(self)?;
            let select_id = self.id_stack.pop().unwrap();
            self.write_edge(&id, &select_id, "select")?;
        }

        self.id_stack.push(id);
        Ok(())
    }

    fn select(&mut self, node: &Select) -> VisitorResult {
        let id = match node.selection {
            Selection::KeepHighest => self.write_node("Keep Highest")?,
            Selection::KeepLowest => self.write_node("Keep Lowest")?,
            Selection::DropHighest => self.write_node("Drop Highest")?,
            Selection::DropLowest => self.write_node("Drop Lowest")?,
            Selection::Advantage => self.write_node("Advantage")?,
            Selection::Disadvantage => self.write_node("Disadvantage")?,
        };

        if let Some(count) = &node.count {
            count.accept(self)?;
            let count_id = self.id_stack.pop().unwrap();
            self.write_edge(&id, &count_id, "count")?;
        }

        if let Some(next) = &node.next {
            next.accept(self)?;
            let select_id = self.id_stack.pop().unwrap();
            self.write_edge(&id, &select_id, "next")?;
        }

        self.id_stack.push(id);
        Ok(())
    }

    fn neg(&mut self, node: &Neg) -> VisitorResult {
        let id = self.write_node("-")?;

        node.right.accept(self)?;
        let right_id = self.id_stack.pop().unwrap();
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }

    fn add(&mut self, node: &Add) -> VisitorResult {
        let id = self.write_node("Add")?;

        node.left.accept(self)?;
        node.right.accept(self)?;
        let left_id = self.id_stack.pop().unwrap();
        let right_id = self.id_stack.pop().unwrap();

        self.write_edge(&id, &left_id, "left")?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }

    fn sub(&mut self, node: &Sub) -> VisitorResult {
        let id = self.write_node("Subtract")?;

        node.left.accept(self)?;
        node.right.accept(self)?;
        let left_id = self.id_stack.pop().unwrap();
        let right_id = self.id_stack.pop().unwrap();

        self.write_edge(&id, &left_id, "left")?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }

    fn mul(&mut self, node: &Mul) -> VisitorResult {
        let id = self.write_node("Multiply")?;

        node.left.accept(self)?;
        node.right.accept(self)?;
        let left_id = self.id_stack.pop().unwrap();
        let right_id = self.id_stack.pop().unwrap();

        self.write_edge(&id, &left_id, "left")?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }

    fn div(&mut self, node: &Div) -> VisitorResult {
        let id = self.write_node("Divide")?;

        node.left.accept(self)?;
        node.right.accept(self)?;
        let left_id = self.id_stack.pop().unwrap();
        let right_id = self.id_stack.pop().unwrap();

        self.write_edge(&id, &left_id, "left")?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }
}
