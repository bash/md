use crate::style::{StyleExt as _, StyledStr};
use crate::textwrap::DisplayWidth;
use anstyle::Style;
use std::cell::RefCell;
use std::fmt::{self, Debug, Display};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub(crate) struct Prefix {
    first: RefCell<Option<StyledStr<'static>>>,
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
            first: RefCell::new(None),
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

    pub(crate) fn is_empty(&self) -> bool {
        let first = self.first.borrow();
        let is_first_empty = first.is_none() || first.as_ref().is_some_and(|f| f.is_empty());
        is_first_empty && self.rest.is_empty()
    }

    fn with_first_special(mut self, value: impl Into<StyledStr<'static>>) -> Self {
        self.first = RefCell::new(Some(value.into()));
        self
    }
}

#[derive(Debug)]
pub(crate) enum PrefixChain<'a> {
    Start(Option<Prefix>, Style),
    Link(&'a PrefixChain<'a>, Prefix, Style, usize),
    Borrowed(&'a PrefixChain<'a>),
}

impl Default for PrefixChain<'_> {
    fn default() -> Self {
        PrefixChain::Start(None, Style::new())
    }
}

impl<'a: 'b, 'b> PrefixChain<'a> {
    pub(crate) fn link(&'a self, prefix: Prefix, style: Style) -> PrefixChain<'a> {
        let width = self.width() + prefix.width();
        let style = style.on_top_of(self.style());
        match self {
            PrefixChain::Start(None, _) => PrefixChain::Start(Some(prefix), style),
            PrefixChain::Start(Some(_), _) => PrefixChain::Link(self, prefix, style, width),
            PrefixChain::Link(..) => PrefixChain::Link(self, prefix, style, width),
            PrefixChain::Borrowed(left) => PrefixChain::Link(left, prefix, style, width),
        }
    }

    pub(crate) fn reborrow(&'b self) -> PrefixChain<'b> {
        if let PrefixChain::Borrowed(inner) = self {
            PrefixChain::Borrowed(inner)
        } else {
            PrefixChain::Borrowed(self)
        }
    }

    pub(crate) fn display_next(&'a self) -> DisplayPrefixChain<'a> {
        DisplayPrefixChain(self)
    }

    fn style(&self) -> Style {
        match self {
            PrefixChain::Start(_, style) => *style,
            PrefixChain::Link(_, _, style, _) => *style,
            PrefixChain::Borrowed(b) => b.style(),
        }
    }
}

impl UnicodeWidthStr for PrefixChain<'_> {
    fn width(&self) -> usize {
        match self {
            PrefixChain::Start(None, _) => 0,
            PrefixChain::Start(Some(start), _) => start.width(),
            PrefixChain::Link(_, _, _, width) => *width,
            PrefixChain::Borrowed(inner) => inner.width(),
        }
    }

    fn width_cjk(&self) -> usize {
        unimplemented!("currently not needed")
    }
}

pub(crate) struct DisplayPrefixChain<'a>(&'a PrefixChain<'a>);

impl fmt::Display for DisplayPrefixChain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            PrefixChain::Start(None, _) => Ok(()),
            PrefixChain::Start(Some(prefix), style) => {
                write!(f, "{}", prefix.take_next().on_top_of(*style))
            }
            PrefixChain::Link(chain, prefix, style, _) => {
                write!(f, "{}", chain.display_next())?;
                write!(f, "{}", prefix.take_next().on_top_of(*style))
            }
            PrefixChain::Borrowed(chain) => Display::fmt(&DisplayPrefixChain(chain), f),
        }
    }
}
