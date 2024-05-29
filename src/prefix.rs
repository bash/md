use crate::textwrap::DisplayWidth;
use std::borrow::{Borrow, Cow};
use std::iter::Sum;
use std::ops::Add;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub(crate) struct Prefix {
    first: Option<Cow<'static, str>>,
    rest: DisplayWidth<Cow<'static, str>>,
}

impl Prefix {
    pub(crate) fn uniform(value: impl Into<Cow<'static, str>>) -> Self {
        Self {
            first: None,
            rest: DisplayWidth::from(value.into()),
        }
    }

    /// A prefix where the first line is special
    /// and the rest is indented with spaces to line up with the first line.
    // TODO: better name
    pub(crate) fn continued(value: impl Into<Cow<'static, str>>) -> Self {
        let value = value.into();
        Self::uniform(" ".repeat(value.width())).with_first_special(value)
    }

    pub(crate) fn with_first_special(mut self, value: impl Into<Cow<'static, str>>) -> Self {
        self.first = Some(value.into());
        self
    }

    pub(crate) fn measure(&self) -> PrefixMeasurement {
        let rest = self.rest.width();
        PrefixMeasurement(
            self.first.as_deref().map(|v| v.width()).unwrap_or(rest),
            rest,
        )
    }

    pub(crate) fn take_next(&mut self) -> Cow<'_, str> {
        if let Some(first) = self.first.take() {
            first
        } else {
            Cow::Borrowed(self.rest.borrow())
        }
    }
}

pub(crate) struct PrefixMeasurement(usize, usize);

impl PrefixMeasurement {
    pub(crate) const fn zero() -> Self {
        PrefixMeasurement(0, 0)
    }

    pub(crate) fn first(&self) -> usize {
        self.0
    }

    pub(crate) fn rest(&self) -> usize {
        self.1
    }
}

impl Add for PrefixMeasurement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        PrefixMeasurement(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sum for PrefixMeasurement {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(PrefixMeasurement::zero(), Add::add)
    }
}
