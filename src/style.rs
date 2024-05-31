use anstyle::{Reset, Style};
use pulldown_cmark::CowStr;
use std::borrow::Borrow;
use std::ops::Deref;
use std::{fmt, mem};

#[derive(Debug)]
pub(crate) struct StyleStack {
    head: Style,
    tail: Vec<Style>,
}

impl StyleStack {
    pub(crate) fn new(head: Style) -> Self {
        Self {
            head,
            tail: Vec::default(),
        }
    }

    pub(crate) fn head(&self) -> Style {
        self.head
    }

    pub(crate) fn push(&mut self, style: Style) {
        self.tail.push(mem::replace(&mut self.head, style));
    }

    pub(crate) fn pop(&mut self) {
        if let Some(new_head) = self.tail.pop() {
            self.head = new_head;
        } else if cfg!(debug_assertions) {
            panic!("stack empty");
        }
    }
}

#[derive(Debug)]
pub(crate) struct StyledStr<'a>(pub(crate) CowStr<'a>, pub(crate) Style);

impl<'a> StyledStr<'a> {
    pub(crate) fn on_top_of(self, style: Style) -> Self {
        Self(self.0, self.1.on_top_of(&style))
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
        write!(f, "{}{}{}", self.1, self.0, Reset)
    }
}

pub(crate) trait StyleExt {
    // TODO: take style by value (it's copy)
    fn on_top_of(&self, fallback: &Style) -> Style;
}

impl StyleExt for Style {
    fn on_top_of(&self, fallback: &Style) -> Style {
        Style::new()
            .effects(self.get_effects() | fallback.get_effects())
            .fg_color(self.get_fg_color().or_else(|| fallback.get_fg_color()))
            .bg_color(self.get_bg_color().or_else(|| fallback.get_bg_color()))
            .underline_color(
                self.get_underline_color()
                    .or_else(|| fallback.get_underline_color()),
            )
    }
}
