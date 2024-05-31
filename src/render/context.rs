use crate::prefix::Prefix;
use crate::style::{StyleExt, StyledStr};
use anstyle::Style;
use pulldown_cmark::HeadingLevel;
use std::cell::Cell;
use std::ops::Deref;
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub(super) enum BlockContext<'a> {
    Inherited(&'a BlockContext<'a>),
    Nested(&'a BlockContext<'a>, BlockContextData),
    Bottom(BlockContextData),
}

impl Deref for BlockContext<'_> {
    type Target = BlockContextData;

    fn deref(&self) -> &Self::Target {
        match self {
            BlockContext::Inherited(p) => p,
            BlockContext::Nested(_, data) => data,
            BlockContext::Bottom(data) => data,
        }
    }
}

impl<'a> BlockContext<'a> {
    pub(super) fn inherited<'b>(&'b self) -> BlockContext<'b> {
        BlockContext::Inherited(self.as_ref())
    }

    pub(super) fn nested<'b>(
        &'b self,
        f: impl FnOnce(BlockContextData) -> BlockContextData,
    ) -> BlockContext<'b> {
        let parent = self.as_ref();
        let data = f(BlockContextData::derive(parent));
        BlockContext::Nested(parent, data)
    }

    // TODO: avoid referencing the parent, find different solution for prefixes.
    pub(super) fn parent(&self) -> Option<&Self> {
        match self {
            BlockContext::Inherited(b) => b.parent(),
            BlockContext::Nested(parent, _) => Some(parent),
            BlockContext::Bottom(_) => None,
        }
    }

    pub(super) fn prefix_width(&self) -> usize {
        self.prefix.width() + self.parent().map(|p| p.prefix_width()).unwrap_or_default()
    }

    // Resolves inherited contexts to avoid too much indirection.
    fn as_ref(&self) -> &Self {
        match self {
            BlockContext::Inherited(b) => b,
            _ => self,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct BlockContextData {
    prefix: Prefix,
    style: Style,
    previous_block: Cell<Option<BlockKind>>,
    current_block: Cell<Option<BlockKind>>,
    list_depth: usize,
}

impl BlockContextData {
    fn derive(parent: &BlockContextData) -> Self {
        BlockContextData {
            prefix: Prefix::default(),
            style: parent.style,
            previous_block: Cell::default(),
            current_block: Cell::default(),
            list_depth: parent.list_depth,
        }
    }
}

impl BlockContextData {
    pub(super) fn prefixed(mut self, prefix: Prefix) -> Self {
        self.prefix = prefix;
        self
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

impl BlockContextData {
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

    pub(super) fn take_prefix(&self) -> StyledStr<'_> {
        self.prefix.take_next()
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
