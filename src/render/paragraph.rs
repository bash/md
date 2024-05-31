use super::context::{BlockContext, BlockKind};
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
        state: &mut State<'e>,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()> {
        let writer = w.inline_writer(state, b);
        writer.write_all(
            terminated!(events, Event::End(TagEnd::Paragraph))
                .flat_map(|event| into_inlines(event, state)),
        )
    }
}
