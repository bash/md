use super::context::{BlockContext, BlockKind};
use super::{prelude::*, BlockRenderer};
use crate::render::inline::into_inlines;

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
        let mut writer = w.inline_writer(state, b);

        take! {
            for event in events; until Event::End(TagEnd::Paragraph) => {
                writer.write_iter(into_inlines(event, state))?;
            }
        }

        writer.end()
    }
}
