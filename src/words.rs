pub struct Words<'a, I, W: 'a>
where
    I: Iterator<Item = &'a W>,
{
    pub(crate) iter: I,
}

impl<'a, I, W> Iterator for Words<'a, I, W>
where
    I: Iterator<Item = &'a W>,
{
    type Item = &'a W;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
