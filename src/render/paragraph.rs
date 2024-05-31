use super::{prelude::*, BlockRenderer};
use crate::inline::into_inlines;

pub(super) struct Paragraph;

impl BlockRenderer for Paragraph {
    fn kind(&self) -> BlockKind {
        BlockKind::Paragraph
    }

    fn render<'e>(
        self,
        events: Events<'_, 'e, '_>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut Writer,
    ) -> io::Result<()> {
        let writer = w.inline_writer(ctx);
        writer.write_all(
            terminated!(events, Event::End(TagEnd::Paragraph))
                .flat_map(|event| into_inlines(event, ctx)),
        )
    }
}
