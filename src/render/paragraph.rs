use super::RenderState;
use crate::fragment::Fragments;
use crate::render::fragment::FragmentsExt as _;
use pulldown_cmark::{Event, TagEnd};
use std::io;

pub(super) fn paragraph(
    events: &mut dyn Iterator<Item = Event<'_>>,
    state: &mut RenderState,
) -> io::Result<()> {
    let mut fragments = Fragments::default();

    take! {
        for event in events; until Event::End(TagEnd::Paragraph) => {
            fragments.try_push_event(event, state);
        }
    }

    state.write_block_start()?;
    state.write_fragments(fragments, state.style())
}
