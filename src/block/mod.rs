use crate::context::{BlockKind, Context};
use crate::writer::Writer;
use crate::Events;
use pulldown_cmark::Event;
use std::io;

mod event;
pub(crate) use event::*;

pub(crate) fn render_block<'e, B: Block>(
    block: B,
    events: Events<'_, 'e, '_>,
    ctx: &Context<'_, 'e, '_>,
    writer: &mut Writer,
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
        events: Events<'_, 'e, '_>,
        ctx: &Context<'_, 'e, '_>,
        writer: &mut Writer,
    ) -> io::Result<()>;

    fn is_blank(&self, _ctx: &Context) -> bool {
        false
    }
}

fn is_blank(block: &impl Block, kind: BlockKind, events: Events, ctx: &Context) -> bool {
    let mut events = events.lookahead();
    block.is_blank(ctx)
        || (has_start_and_end_tag(kind) && matches!(events.next(), Some(Event::End(_))))
}

fn has_start_and_end_tag(kind: BlockKind) -> bool {
    !matches!(kind, BlockKind::Rule)
}
