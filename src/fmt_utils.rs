use std::fmt;

pub(crate) struct Repeat<T>(pub(crate) usize, pub(crate) T);

impl<T> fmt::Display for Repeat<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (0..self.0).try_for_each(|_| self.1.fmt(f))
    }
}
