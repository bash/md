use crate::textwrap::RawChunk;
use anstyle::Style;
use pulldown_cmark::CowStr;
use url::Url;

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

impl<'a> From<Inline<'a>> for RawChunk<'a, Inline<'a>> {
    fn from(value: Inline<'a>) -> Self {
        match value {
            Inline::Text(text) => RawChunk::Text(text),
            Inline::SoftBreak => RawChunk::soft_break(),
            Inline::HardBreak => RawChunk::hard_break(),
            other => RawChunk::Passthrough(other),
        }
    }
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
