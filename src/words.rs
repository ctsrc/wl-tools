/// An iterator over words `W`
pub struct Words<'a, W: 'a>(Box<dyn Iterator<Item = &'a W> + 'a>);

impl<'a, W> Words<'a, W> {
    pub fn new(inner_boxed: Box<dyn Iterator<Item = &'a W> + 'a>) -> Self {
        Self(inner_boxed)
    }
}

impl<'a, W> Iterator for Words<'a, W> {
    type Item = &'a W;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
