use super::prelude::*;
use crate::render::inline::into_inlines;

pub(super) fn paragraph(events: Events, state: &mut State, w: &mut Writer) -> io::Result<()> {
    w.write_block_start()?;

    let mut writer = w.inline_writer(state);

    take! {
        for event in events; until Event::End(TagEnd::Paragraph) => {
            writer.write_iter(into_inlines(event, state))?;
        }
    }

    writer.end()
}
