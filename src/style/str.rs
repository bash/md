use crate::style::StyleExt as _;
use anstyle::{Reset, Style};
use pulldown_cmark::CowStr;
use std::borrow::Borrow as _;
use std::fmt;
use std::ops::Deref;

#[derive(Debug)]
pub(crate) struct StyledStr<'a>(pub(crate) CowStr<'a>, pub(crate) Style);

impl<'a> StyledStr<'a> {
    pub(crate) fn on_top_of(self, fallback: Style) -> Self {
        Self(self.0, self.1.on_top_of(fallback))
    }
}

impl<'a> StyledStr<'a> {
    pub(crate) fn new(s: impl Into<CowStr<'a>>, style: Style) -> Self {
        Self(s.into(), style)
    }
}

impl StyledStr<'_> {
    pub(crate) fn borrowed(&self) -> StyledStr<'_> {
        StyledStr(CowStr::Borrowed(self.0.borrow()), self.1)
    }
}

impl Default for StyledStr<'_> {
    fn default() -> Self {
        Self::new("", Default::default())
    }
}

impl Deref for StyledStr<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a str> for StyledStr<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value, Style::default())
    }
}

impl<'a> From<String> for StyledStr<'a> {
    fn from(value: String) -> Self {
        Self::new(value, Style::default())
    }
}

impl<'a> From<CowStr<'a>> for StyledStr<'a> {
    fn from(value: CowStr<'a>) -> Self {
        Self(value, Style::default())
    }
}

impl fmt::Display for StyledStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.1.is_plain() {
            write!(f, "{}", self.0)
        } else {
            write!(f, "{}{}{}", self.1, self.0, Reset)
        }
    }
}
