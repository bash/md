use crate::fmt_utils::NoDebug;
use crate::style::StyledStr;
use crate::textwrap::DisplayWidth;
use std::cell::Cell;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub(crate) struct Prefix {
    first: NoDebug<Cell<Option<StyledStr<'static>>>>,
    rest: DisplayWidth<StyledStr<'static>>,
}

impl UnicodeWidthStr for Prefix {
    fn width(&self) -> usize {
        self.rest.width()
    }

    fn width_cjk(&self) -> usize {
        self.rest.width_cjk()
    }
}

impl Prefix {
    pub(crate) fn uniform(value: impl Into<StyledStr<'static>>) -> Self {
        Self {
            first: NoDebug(Cell::new(None)),
            rest: DisplayWidth::from(value.into()),
        }
    }

    /// A prefix where the first line is special
    /// and the rest is indented with spaces to line up with the first line.
    pub(crate) fn continued(value: impl Into<StyledStr<'static>>) -> Self {
        let value = value.into();
        let repeated = " ".repeat(value.0.width());
        Self::uniform(StyledStr(repeated.into(), value.1)).with_first_special(value)
    }

    pub(crate) fn take_next(&self) -> StyledStr<'_> {
        if let Some(first) = self.first.take() {
            first
        } else {
            self.rest.value().borrowed()
        }
    }

    fn with_first_special(mut self, value: impl Into<StyledStr<'static>>) -> Self {
        self.first = NoDebug(Cell::new(Some(value.into())));
        self
    }
}
