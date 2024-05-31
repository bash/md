use crate::prefix::{Prefix, PrefixChain};
use crate::style::StyleExt;
use anstyle::Style;
use pulldown_cmark::HeadingLevel;
use std::cell::Cell;

#[derive(Debug, Default)]
pub(super) struct BlockContext<'a> {
    prefix: PrefixChain<'a>,
    style: Style,
    previous_block: Cell<Option<BlockKind>>,
    current_block: Cell<Option<BlockKind>>,
    list_depth: usize,
}

impl<'a> BlockContext<'a> {
    pub(super) fn child(&self, prefix: impl Into<Option<Prefix>>) -> BlockContext<'_> {
        let prefix = match prefix.into() {
            Some(p) => self.prefix.link(p),
            None => self.prefix.reborrow(),
        };
        BlockContext {
            prefix,
            style: self.style,
            previous_block: Cell::default(),
            current_block: Cell::default(),
            list_depth: self.list_depth,
        }
    }

    pub(super) fn styled(mut self, style: Style) -> Self {
        self.style = style.on_top_of(&self.style);
        self
    }

    pub(super) fn list_depth_incremented(mut self) -> Self {
        self.list_depth += 1;
        self
    }
}

impl BlockContext<'_> {
    pub(super) fn prefix_chain(&self) -> &PrefixChain {
        &self.prefix
    }

    pub(super) fn style(&self) -> Style {
        self.style
    }

    pub(super) fn previous_block(&self) -> Option<BlockKind> {
        self.previous_block.get()
    }

    pub(super) fn set_previous_block(&self, b: BlockKind) {
        self.previous_block.set(Some(b));
    }

    pub(super) fn current_block(&self) -> Option<BlockKind> {
        self.current_block.get()
    }

    pub(super) fn set_current_block(&self, b: BlockKind) {
        self.current_block.set(Some(b));
    }

    pub(super) fn list_depth(&self) -> usize {
        self.list_depth
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
