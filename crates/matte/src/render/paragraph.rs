use super::prelude::*;
use crate::block::Block;
use crate::inline::into_inlines;

pub(crate) struct Paragraph;

impl Block for Paragraph {
    fn kind(&self) -> BlockKind {
        BlockKind::Paragraph
    }

    fn render<'e>(
        self,
        events: &mut impl Events<'e>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut impl Write,
    ) -> io::Result<()> {
        let writer = w.inline_writer(ctx);
        writer.write_all(
            terminated!(events, Event::End(TagEnd::Paragraph))
                .flat_map(|event| into_inlines(event, ctx)),
        )
    }
}
