use core::fmt;
use std::cell::OnceCell;
use std::ops::Deref;
use textwrap::core::display_width;

#[derive(Debug, Default)]
pub(crate) struct DisplayWidth<T> {
    value: T,
    display_width: OnceCell<usize>,
}

impl<T> DisplayWidth<T> {
    pub(crate) fn value(&self) -> &T {
        &self.value
    }
}

impl<T> From<T> for DisplayWidth<T> {
    fn from(value: T) -> Self {
        Self {
            value,
            display_width: OnceCell::new(),
        }
    }
}

impl<'a, T> DisplayWidth<T>
where
    T: Deref<Target = str>,
{
    pub(crate) fn display_width(&self) -> usize {
        *self
            .display_width
            .get_or_init(|| display_width(&self.value))
    }
}

impl<T> Deref for DisplayWidth<T>
where
    T: Deref<Target = str>,
{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> fmt::Display for DisplayWidth<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
