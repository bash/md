use crate::bullets::Bullets;
use crate::counting::Counters;
use crate::footnotes::Footnotes;
use crate::prefix::{Prefix, PrefixChain};
use crate::style::StyleExt;
use crate::Options;
use anstyle::Style;
use pulldown_cmark::HeadingLevel;
use std::cell::Cell;
use std::cmp::min;
use unicode_width::UnicodeWidthStr as _;

#[derive(Debug)]
pub(crate) struct Context<'a, 'e, 's> {
    prefix: PrefixChain<'a>,
    style: Style,
    previous_block: Cell<Option<BlockKind>>,
    current_block: Cell<Option<BlockKind>>,
    list_depth: usize,
    state: &'s State<'e>,
}

impl<'a, 'e, 's> Context<'a, 'e, 's> {
    pub(crate) fn new(state: &'s State<'e>) -> Self {
        Self {
            prefix: Default::default(),
            style: Default::default(),
            previous_block: Default::default(),
            current_block: Default::default(),
            list_depth: Default::default(),
            state,
        }
    }
}

#[derive(Debug)]
pub(crate) struct State<'e> {
    options: Options,
    counters: Counters,
    footnotes: Footnotes<'e>,
    bullets: Bullets,
}

impl State<'_> {
    pub(crate) fn new(options: Options) -> Self {
        Self {
            bullets: Bullets::default_for(options.symbol_repertoire),
            options,
            counters: Counters::default(),
            footnotes: Footnotes::default(),
        }
    }
}

impl<'a, 'e, 's> Context<'a, 'e, 's> {
    pub(crate) fn block<'b: 'a>(
        &'b self,
        prefix: impl Into<Option<Prefix>>,
    ) -> Context<'b, 'e, 's> {
        let prefix = match prefix.into() {
            Some(p) => self.prefix.link(p),
            None => self.prefix.reborrow(),
        };
        Self {
            prefix,
            style: self.style,
            previous_block: Cell::default(),
            current_block: Cell::default(),
            list_depth: self.list_depth,
            state: self.state,
        }
    }

    pub(crate) fn styled(mut self, style: Style) -> Self {
        self.style = style.on_top_of(self.style);
        self
    }

    pub(crate) fn list_depth_incremented(mut self) -> Self {
        self.list_depth += 1;
        self
    }
}

impl<'a, 'e> Context<'a, 'e, '_> {
    pub(crate) fn prefix_chain(&self) -> &PrefixChain<'a> {
        &self.prefix
    }

    pub(crate) fn style(&self) -> Style {
        self.style
    }

    pub(crate) fn previous_block(&self) -> Option<BlockKind> {
        self.previous_block.get()
    }

    pub(crate) fn set_previous_block(&self, b: BlockKind) {
        self.previous_block.set(Some(b));
    }

    pub(crate) fn current_block(&self) -> Option<BlockKind> {
        self.current_block.get()
    }

    pub(crate) fn set_current_block(&self, b: BlockKind) {
        self.current_block.set(Some(b));
    }

    pub(crate) fn options(&self) -> &Options {
        &self.state.options
    }

    pub(crate) fn footnotes(&self) -> &Footnotes<'e> {
        &self.state.footnotes
    }

    pub(crate) fn available_width(&self) -> usize {
        (self.options().columns as usize) - self.prefix.width()
    }

    pub(crate) fn text_width(&self) -> usize {
        min(self.available_width(), self.options().text_max_columns)
    }

    pub(crate) fn counters(&self) -> &Counters {
        &self.state.counters
    }

    pub(crate) fn bullet(&self) -> &str {
        self.state.bullets.nth(self.list_depth)
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum BlockKind {
    Heading(HeadingLevel),
    Paragraph,
    CodeBlock,
    BlockQuote,
    List,
    Rule,
    Table,
    FootnoteDefinition,
}
