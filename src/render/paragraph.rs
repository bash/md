use super::prelude::*;
use crate::render::fragment::into_fragments;

pub(super) fn paragraph(events: Events, state: &mut State, w: &mut Writer) -> io::Result<()> {
    w.write_block_start()?;

    let mut writer = w.fragment_writer(state);

    take! {
        for event in events; until Event::End(TagEnd::Paragraph) => {
            writer.write_iter(into_fragments(event))?;
        }
    }

    writer.end()
}
