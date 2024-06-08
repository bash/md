use super::*;
use crate::counting::SectionCounter;
use anstyle::AnsiColor::{Blue, Green};
use std::fmt::Write as _;

#[derive(Debug)]
pub(super) struct DefaultTheme;

impl ThemeProvider for DefaultTheme {
    fn margin_size(&self, _a: &BlockKind, _b: &BlockKind, _ctx: &Context<'_, '_, '_>) -> usize {
        1
    }

    fn block_quote_style(
        &self,
        _kind: Option<block_quote::Kind>,
        _ctx: &Context<'_, '_, '_>,
    ) -> Style {
        Style::default()
    }

    fn block_quote_prefix(
        &self,
        kind: Option<block_quote::Kind>,
        _ctx: &Context<'_, '_, '_>,
    ) -> Prefix {
        let style = kind.map(|k| k.style()).unwrap_or_default();
        Prefix::uniform(StyledStr::new("┃ ", style))
    }

    fn heading_style(&self, level: HeadingLevel, _ctx: &Context<'_, '_, '_>) -> Style {
        match level {
            HeadingLevel::H1 => Green.on_default().bold().underline(),
            HeadingLevel::H2 => Green.on_default().bold(),
            _ => Blue.on_default(),
        }
    }

    fn heading_prefix(&self, _level: HeadingLevel, ctx: &Context<'_, '_, '_>) -> Prefix {
        Prefix::continued(numbering(ctx.counters().section()))
    }
}

// TODO: having numbering for changelog files is really not nice
// Since this needs to be configurable anyways maybe we can have a heuristic
// that detects changelog files by name (any or no extension):
// * changelog, CHANGELOG, RELEASE_NOTES, releasenotes, RELEASENOTES
// others?
fn numbering(counters: SectionCounter) -> String {
    let mut output = String::new();
    let numbers = &counters.as_slice()[1..];

    // No numbering for sections with leading zeroes.
    if !numbers.is_empty() && !numbers.starts_with(&[0]) {
        for n in numbers {
            write!(output, "{n}.").unwrap(); // TODO
        }
        write!(output, " ").unwrap(); // TODO
    }

    output
}
