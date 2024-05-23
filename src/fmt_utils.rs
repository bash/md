use std::fmt;
use std::ops::{Deref, DerefMut};

pub(crate) struct Repeat<T>(pub(crate) usize, pub(crate) T);

impl<T> fmt::Display for Repeat<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (0..self.0).try_for_each(|_| self.1.fmt(f))
    }
}

pub(crate) struct NoDebug<T>(T);

impl<T> From<T> for NoDebug<T> {
    fn from(value: T) -> Self {
        NoDebug(value)
    }
}

impl<T> fmt::Debug for NoDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "?")
    }
}

impl<T> Deref for NoDebug<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NoDebug<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
