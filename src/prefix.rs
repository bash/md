use crate::fmt_utils::NoDebug;
use crate::style::StyledStr;
use crate::textwrap::DisplayWidth;
use anstyle::Style;
use std::cell::Cell;
use std::fmt::{self, Debug};
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

#[derive(Debug)]
pub(crate) enum PrefixChain<'a> {
    Start(Option<Prefix>),
    Link(&'a PrefixChain<'a>, Prefix, usize),
    Borrowed(&'a PrefixChain<'a>),
}

impl Default for PrefixChain<'_> {
    fn default() -> Self {
        PrefixChain::Start(None)
    }
}

impl<'a: 'b, 'b> PrefixChain<'a> {
    pub(crate) fn link(&'a self, prefix: Prefix) -> PrefixChain<'a> {
        let width = self.width() + prefix.width();
        match self {
            PrefixChain::Start(None) => PrefixChain::Start(Some(prefix)),
            PrefixChain::Start(Some(_)) => PrefixChain::Link(self, prefix, width),
            PrefixChain::Link(..) => PrefixChain::Link(self, prefix, width),
            PrefixChain::Borrowed(left) => PrefixChain::Link(left, prefix, width),
        }
    }

    pub(crate) fn reborrow(&'b self) -> PrefixChain<'b> {
        if let PrefixChain::Borrowed(inner) = self {
            PrefixChain::Borrowed(inner)
        } else {
            PrefixChain::Borrowed(self)
        }
    }

    pub(crate) fn display_next(&'a self, fallback: Style) -> DisplayPrefixChain<'a> {
        DisplayPrefixChain(self, fallback)
    }
}

impl UnicodeWidthStr for PrefixChain<'_> {
    fn width(&self) -> usize {
        match self {
            PrefixChain::Start(None) => 0,
            PrefixChain::Start(Some(start)) => start.width(),
            PrefixChain::Link(_, _, width) => *width,
            PrefixChain::Borrowed(inner) => inner.width(),
        }
    }

    fn width_cjk(&self) -> usize {
        unimplemented!("currently not needed")
    }
}

pub(crate) struct DisplayPrefixChain<'a>(&'a PrefixChain<'a>, Style);

impl fmt::Display for DisplayPrefixChain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            PrefixChain::Start(None) => Ok(()),
            PrefixChain::Start(Some(prefix)) => {
                write!(f, "{}", prefix.take_next().on_top_of(self.1))
            }
            PrefixChain::Link(chain, prefix, _) => {
                write!(f, "{}", chain.display_next(self.1))?;
                write!(f, "{}", prefix.take_next().on_top_of(self.1))
            }
            PrefixChain::Borrowed(chain) => chain.fmt(f),
        }
    }
}
