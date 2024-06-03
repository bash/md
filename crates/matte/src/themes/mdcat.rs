use super::*;
use anstyle::AnsiColor::Blue;

#[derive(Debug)]
pub(super) struct MdcatTheme;

impl ThemeProvider for MdcatTheme {
    fn margin_size(&self, _a: &BlockKind, _b: &BlockKind, _ctx: &Context) -> usize {
        1
    }

    fn block_quote_style(&self, _kind: Option<block_quote::Kind>, _ctx: &Context) -> Style {
        Style::new().italic()
    }

    fn block_quote_prefix(&self, _kind: Option<block_quote::Kind>, _ctx: &Context) -> Prefix {
        Prefix::uniform("    ")
    }

    fn heading_style(&self, _level: HeadingLevel, _ctx: &Context) -> Style {
        Blue.on_default().bold()
    }

    fn heading_prefix(&self, level: HeadingLevel, _ctx: &Context) -> Prefix {
        match level {
            HeadingLevel::H1 => Prefix::continued("┈"),
            HeadingLevel::H2 => Prefix::continued("┈┈"),
            HeadingLevel::H3 => Prefix::continued("┈┈┈"),
            HeadingLevel::H4 => Prefix::continued("┈┈┈┈"),
            HeadingLevel::H5 => Prefix::continued("┈┈┈┈┈"),
            HeadingLevel::H6 => Prefix::continued("┈┈┈┈┈┈"),
        }
    }
}
