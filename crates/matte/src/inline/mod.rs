use anstyle::Style;
use pulldown_cmark::CowStr;
use url::Url;

mod event;
pub(crate) use event::*;
mod hyperlink;
mod writer;
pub(crate) use writer::*;

#[derive(Debug)]
pub(crate) enum Inline<'a> {
    Text(CowStr<'a>),
    SoftBreak,
    HardBreak,
    PushStyle(Style),
    PopStyle,
    SetLink(Url),
    UnsetLink,
}

impl<'a> From<CowStr<'a>> for Inline<'a> {
    fn from(value: CowStr<'a>) -> Self {
        Inline::Text(value)
    }
}

impl<'a> From<&'a str> for Inline<'a> {
    fn from(value: &'a str) -> Self {
        Inline::Text(CowStr::from(value))
    }
}

impl From<Style> for Inline<'_> {
    fn from(value: Style) -> Self {
        Inline::PushStyle(value)
    }
}
