// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

//! Components for writing the AST to a visual graph. The graph can be written
//! in either [Graphviz DOT language](https://graphviz.org/) or
//! [Mermaid language](https://mermaid.js.org/).

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Error as IoError, Write},
    result::Result,
};

use crate::ast::{Node, Selection};

/// Possible errors that can occur during processing.
#[derive(Debug)]
pub enum Error {
    /// The stack was empty when an operation expected a value.
    StackUnderflow,

    /// An I/O error occurred while writing the graph.
    Io(IoError),
}

/// The language in which to write the graph.
enum GraphLang {
    /// The Graphviz DOT language (see https://graphviz.org/).
    Dot,

    /// Mermaid language (see https://mermaid.js.org/).
    Mermaid,
}

/// Writes a graph of the AST to a writer in the Graphviz DOT or Mermaid format.
pub struct GraphWriter<'o, W: Write> {
    /// The writer to which the pretty-printed expression is written.
    writer: &'o mut W,

    /// The unique identifier for the next node to be written.
    next_id: usize,

    /// A stack of node identifiers to track the current path through the AST.
    id_stack: Vec<String>,

    /// The language in which to write the graph.
    lang: GraphLang,
}

impl<'o, W: Write> GraphWriter<'o, W> {
    pub fn new_dot(writer: &'o mut W) -> Self {
        Self {
            writer,
            next_id: 1,
            id_stack: Vec::new(),
            lang: GraphLang::Dot,
        }
    }

    pub fn new_mermaid(writer: &'o mut W) -> Self {
        Self {
            writer,
            next_id: 1,
            id_stack: Vec::new(),
            lang: GraphLang::Mermaid,
        }
    }

    pub fn write(&mut self, root: &Node) -> Result<(), Error> {
        match self.lang {
            GraphLang::Dot => {
                writeln!(self.writer, "graph {{")?;
                writeln!(self.writer, "    graph [rankdir=TB]")?;
                writeln!(self.writer, "    node [shape=rect]")?;
                writeln!(self.writer, "    edge [fontsize=10]")?;
            }
            GraphLang::Mermaid => {
                writeln!(self.writer, "graph TB")?;
            }
        }

        self.visit(root)?;

        match self.lang {
            GraphLang::Dot => {
                writeln!(self.writer, "}}")?;
            }
            GraphLang::Mermaid => {}
        }

        Ok(())
    }

    fn write_node(&mut self, label: &str) -> Result<String, IoError> {
        let id = format!("node{:04x}", self.next_id);
        self.next_id += 1;
        match self.lang {
            GraphLang::Dot => writeln!(self.writer, "    {id} [label=\"{label}\"]")?,
            GraphLang::Mermaid => writeln!(self.writer, "    {id}(\"{label}\")")?,
        }
        Ok(id)
    }

    fn write_edge(&mut self, parent: &str, child: &str, label: &str) -> Result<(), IoError> {
        match self.lang {
            GraphLang::Dot => writeln!(self.writer, "    {parent} -- {child} [label=\"{label}\"]")?,
            GraphLang::Mermaid => writeln!(self.writer, "    {parent} --{label}--- {child}",)?,
        }
        Ok(())
    }

    fn visit(&mut self, node: &Node) -> Result<(), Error> {
        match node {
            Node::Lit { value } => self.lit(*value),
            Node::Roll {
                count,
                sides,
                select,
            } => self.roll(
                count.as_ref(),
                sides.as_ref(),
                select.as_ref().map(|n| n.as_ref()),
            ),
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
        let id = self.write_node(&format!("{}", value))?;
        self.id_stack.push(id);
        Ok(())
    }

    fn roll(&mut self, count: &Node, sides: &Node, select: Option<&Node>) -> Result<(), Error> {
        let id = self.write_node("Roll")?;
        self.visit(count)?;
        let count_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.visit(sides)?;
        let sides_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.write_edge(&id, &count_id, "count")?;
        self.write_edge(&id, &sides_id, "sides")?;

        if let Some(selection) = &select {
            self.visit(selection)?;
            let select_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;
            self.write_edge(&id, &select_id, "select")?;
        }

        self.id_stack.push(id);
        Ok(())
    }

    fn select(&mut self, selection: &Selection, next: Option<&Node>) -> Result<(), Error> {
        let id = match selection {
            Selection::KeepHighest { .. } => self.write_node("Keep Highest")?,
            Selection::KeepLowest { .. } => self.write_node("Keep Lowest")?,
            Selection::DropHighest { .. } => self.write_node("Drop Highest")?,
            Selection::DropLowest { .. } => self.write_node("Drop Lowest")?,
            Selection::Advantage => self.write_node("Advantage")?,
            Selection::Disadvantage => self.write_node("Disadvantage")?,
        };

        match selection {
            Selection::KeepHighest { count }
            | Selection::KeepLowest { count }
            | Selection::DropHighest { count }
            | Selection::DropLowest { count } => {
                if let Some(count) = &count {
                    self.visit(count)?;
                    let count_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;
                    self.write_edge(&id, &count_id, "count")?;
                }
            }
            _ => {}
        };

        if let Some(next) = &next {
            self.visit(next)?;
            let select_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;
            self.write_edge(&id, &select_id, "next")?;
        }

        self.id_stack.push(id);
        Ok(())
    }

    fn neg(&mut self, right: &Node) -> Result<(), Error> {
        let id = self.write_node("-")?;

        self.visit(right)?;
        let right_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }

    fn add(&mut self, left: &Node, right: &Node) -> Result<(), Error> {
        let id = self.write_node("Add")?;

        self.visit(left)?;
        let left_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.visit(right)?;
        let right_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.write_edge(&id, &left_id, "left")?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }

    fn sub(&mut self, left: &Node, right: &Node) -> Result<(), Error> {
        let id = self.write_node("Subtract")?;

        self.visit(left)?;
        let left_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.visit(right)?;
        let right_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.write_edge(&id, &left_id, "left")?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }

    fn mul(&mut self, left: &Node, right: &Node) -> Result<(), Error> {
        let id = self.write_node("Multiply")?;

        self.visit(left)?;
        let left_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.visit(right)?;
        let right_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.write_edge(&id, &left_id, "left")?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }

    fn div(&mut self, left: &Node, right: &Node) -> Result<(), Error> {
        let id = self.write_node("Divide")?;

        self.visit(left)?;
        let left_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.visit(right)?;
        let right_id = self.id_stack.pop().ok_or(Error::StackUnderflow)?;

        self.write_edge(&id, &left_id, "left")?;
        self.write_edge(&id, &right_id, "right")?;

        self.id_stack.push(id);
        Ok(())
    }
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::StackUnderflow => write!(f, "Stack underflow"),
            Error::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}
