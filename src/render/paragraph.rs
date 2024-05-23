use super::Context;
use crate::fragment::Fragments;
use crate::render::fragment::FragmentsExt as _;
use pulldown_cmark::{Event, TagEnd};
use std::io;

pub(super) fn paragraph(
    events: &mut dyn Iterator<Item = Event<'_>>,
    ctx: &mut Context,
) -> io::Result<()> {
    let mut fragments = Fragments::default();

    take! {
        for event in events; until Event::End(TagEnd::Paragraph) => {
            fragments.try_push_event(event, ctx);
        }
    }

    ctx.write_block_start()?;
    ctx.write_fragments(fragments)
}
