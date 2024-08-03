pub struct Lookahead<TIter: Iterator<Item: Copy>>{
    iter: TIter,
    peek: Option<TIter::Item>,
}

impl<TIter: Iterator<Item: Copy>> Lookahead<TIter> {
    pub fn new(iter: TIter) -> Self {
        let mut lookahead = Lookahead {
            iter,
            peek: None,
        };
        lookahead.next();
        lookahead
    }

    pub fn peek(&self) -> Option<&TIter::Item> {
        self.peek.as_ref()
    }

    pub fn next(&mut self) -> Option<TIter::Item> {
        let next = self.iter.next();
        self.peek = next;
        next
    }
}
