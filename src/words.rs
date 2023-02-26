pub struct Words<I, W>
where
    I: Iterator<Item = W>,
{
    pub(crate) iter: I,
}
