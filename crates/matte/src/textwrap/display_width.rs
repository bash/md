use core::fmt;
use std::cell::OnceCell;
use std::ops::Deref;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default, Eq)]
pub(crate) struct DisplayWidth<T> {
    value: T,
    width: OnceCell<usize>,
}

impl<T> DisplayWidth<T> {
    pub(crate) fn value(&self) -> &T {
        &self.value
    }
}

impl<T> UnicodeWidthStr for DisplayWidth<T>
where
    T: Deref<Target = str>,
{
    fn width(&self) -> usize {
        *self.width.get_or_init(|| self.value.width())
    }

    fn width_cjk(&self) -> usize {
        unimplemented!("currently not needed")
    }
}

impl<T> PartialEq for DisplayWidth<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> From<T> for DisplayWidth<T> {
    fn from(value: T) -> Self {
        Self {
            value,
            width: OnceCell::new(),
        }
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
