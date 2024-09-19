// Copyright 2024 Jonathon Cobb
// Licensed under the ISC license

/// A lookahead over an iterator that allows peeking at the next item without
/// consuming it.
pub struct Lookahead<TIter: Iterator<Item: Clone>> {
    iter: TIter,
    peek: Option<TIter::Item>,
}

impl<TIter: Iterator<Item: Clone>> Lookahead<TIter> {
    pub fn new(iter: TIter) -> Self {
        let mut lookahead = Lookahead { iter, peek: None };
        lookahead.next();
        lookahead
    }

    pub fn peek(&self) -> Option<&TIter::Item> {
        self.peek.as_ref()
    }

    pub fn next(&mut self) -> Option<TIter::Item> {
        let next = self.iter.next();
        self.peek = next.clone();
        next
    }
}
