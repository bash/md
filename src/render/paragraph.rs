use super::{Events, State};
use crate::render::fragment::into_fragments;
use pulldown_cmark::{Event, TagEnd};
use std::io;

pub(super) fn paragraph(events: Events, state: &mut State) -> io::Result<()> {
    state.write_block_start()?;

    let mut writer = state.fragment_writer();

    take! {
        for event in events; until Event::End(TagEnd::Paragraph) => {
            writer.write_iter(into_fragments(event))?;
        }
    }

    writer.end()
}
