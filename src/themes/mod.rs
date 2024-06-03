mod default;
mod mdcat;

use crate::block::BlockKind;
use crate::block_quote;
use crate::context::Context;
use crate::prefix::Prefix;
use crate::style::StyledStr;
use anstyle::Style;
use pulldown_cmark::HeadingLevel;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Theme(Arc<dyn ThemeProvider>);

impl Default for Theme {
    fn default() -> Self {
        Self(Arc::new(default::DefaultTheme))
    }
}

impl Theme {
    pub fn mdcat() -> Theme {
        Self(Arc::new(mdcat::MdcatTheme))
    }
}

pub(crate) trait ThemeProvider: fmt::Debug {
    fn margin_size(&self, a: &BlockKind, b: &BlockKind, ctx: &Context) -> usize;

    fn block_quote_style(&self, kind: Option<block_quote::Kind>, ctx: &Context) -> Style;

    fn block_quote_prefix(&self, kind: Option<block_quote::Kind>, ctx: &Context) -> Prefix;

    fn heading_style(&self, level: HeadingLevel, ctx: &Context) -> Style;

    fn heading_prefix(&self, level: HeadingLevel, ctx: &Context) -> Prefix;
}

impl ThemeProvider for Theme {
    fn margin_size(&self, a: &BlockKind, b: &BlockKind, ctx: &Context) -> usize {
        self.0.margin_size(a, b, ctx)
    }

    fn block_quote_style(&self, kind: Option<block_quote::Kind>, ctx: &Context) -> Style {
        self.0.block_quote_style(kind, ctx)
    }

    fn block_quote_prefix(&self, kind: Option<block_quote::Kind>, ctx: &Context) -> Prefix {
        self.0.block_quote_prefix(kind, ctx)
    }

    fn heading_style(&self, level: HeadingLevel, ctx: &Context) -> Style {
        self.0.heading_style(level, ctx)
    }

    fn heading_prefix(&self, level: HeadingLevel, ctx: &Context) -> Prefix {
        self.0.heading_prefix(level, ctx)
    }
}
