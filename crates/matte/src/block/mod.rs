use prelude::*;
use pulldown_cmark::HeadingLevel;

mod event;
pub(crate) use event::*;

#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
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

/// Useful imports when implementing a [`Block`]
pub(crate) mod prelude {
    pub(crate) use super::{Block, BlockKind};
    pub(crate) use crate::context::Context;
    pub(crate) use crate::lookahead::Lookahead as _;
    pub(crate) use crate::writer::WriteExt as _;
    pub(crate) use crate::Events;
    pub(crate) use pulldown_cmark::{Event, Tag, TagEnd};
    pub(crate) use std::io::{self, Write};
}

pub(crate) fn render_block<'e, B: Block>(
    block: B,
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
    writer: &mut impl Write,
) -> io::Result<()> {
    let kind = block.kind();
    if !is_blank(&block, kind, events, ctx) {
        writer.write_block_start(ctx)?;
    }
    ctx.set_current_block(kind);
    block.render(events, ctx, writer)?;
    ctx.set_previous_block(kind);
    Ok(())
}

pub(crate) trait Block {
    fn kind(&self) -> BlockKind;

    fn render<'e>(
        self,
        events: &mut impl Events<'e>,
        ctx: &Context<'_, 'e, '_>,
        writer: &mut impl Write,
    ) -> io::Result<()>;

    fn is_blank(&self, _ctx: &Context<'_, '_, '_>) -> bool {
        false
    }
}

fn is_blank<'e>(
    block: &impl Block,
    kind: BlockKind,
    events: &mut impl Events<'e>,
    ctx: &Context<'_, '_, '_>,
) -> bool {
    let mut events = events.lookahead();
    block.is_blank(ctx)
        || (has_start_and_end_tag(kind) && matches!(events.next(), Some(Event::End(_))))
}

fn has_start_and_end_tag(kind: BlockKind) -> bool {
    !matches!(kind, BlockKind::Rule)
}
