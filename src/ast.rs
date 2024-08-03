use std::error::Error;

pub trait Node: std::fmt::Debug {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult;
}

pub type VisitorResult = Result<(), Box<dyn Error>>;

pub trait Visitor {
    fn literal(&mut self, node: &Literal) -> VisitorResult;
    fn roll(&mut self, node: &Roll) -> VisitorResult;
    fn select(&mut self, node: &Select) -> VisitorResult;
    fn negate(&mut self, node: &Negate) -> VisitorResult;
    fn add(&mut self, node: &Add) -> VisitorResult;
    fn subtract(&mut self, node: &Subtract) -> VisitorResult;
    fn multiply(&mut self, node: &Multiply) -> VisitorResult;
    fn divide(&mut self, node: &Divide) -> VisitorResult;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Selection {
    KeepLowest,
    KeepHighest,
    DropLowest,
    DropHighest,
    Advantage,
    Disadvantage,
}

#[derive(Debug)]
pub struct Literal {
    pub value: i32,
}

#[derive(Debug)]
pub struct Roll {
    pub count: Box<dyn Node>,
    pub sides: Box<dyn Node>,
    pub select: Option<Box<dyn Node>>,
}

#[derive(Debug)]
pub struct Select {
    pub selection: Selection,
    pub count: Option<Box<dyn Node>>,
    pub next: Option<Box<dyn Node>>,
}

#[derive(Debug)]
pub struct Negate {
    pub right: Box<dyn Node>,
}

#[derive(Debug)]
pub struct Add {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

#[derive(Debug)]
pub struct Subtract {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

#[derive(Debug)]
pub struct Multiply {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

#[derive(Debug)]
pub struct Divide {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

impl Node for Literal {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.literal(self)
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

impl Node for Negate {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.negate(self)
    }
}

impl Node for Add {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.add(self)
    }
}

impl Node for Subtract {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.subtract(self)
    }
}

impl Node for Multiply {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.multiply(self)
    }
}

impl Node for Divide {
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.divide(self)
    }
}
