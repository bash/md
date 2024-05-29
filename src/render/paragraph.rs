use super::{Events, State};
use crate::render::fragment::FragmentWriterExt;
use pulldown_cmark::{Event, TagEnd};
use std::io;

pub(super) fn paragraph(events: Events, state: &mut State) -> io::Result<()> {
    state.write_block_start()?;

    let mut fragments = state.fragment_writer();

    take! {
        for event in events; until Event::End(TagEnd::Paragraph) => {
            fragments.try_write_event(event)?;
        }
    }

    fragments.end()
}
