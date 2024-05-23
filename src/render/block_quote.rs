use super::RenderState;
use crate::render::block;
use pulldown_cmark::{Event, TagEnd};
use std::io;

pub(super) fn block_quote(
    events: &mut dyn Iterator<Item = Event<'_>>,
    state: &mut RenderState,
) -> io::Result<()> {
    state.write_block_start()?;

    let scope = state.scope(state.style(), Some("┃ "));

    take! {
        for event in events; until Event::End(TagEnd::BlockQuote) => {
            block(event, events, state)?;
        }
    }

    state.end_scope(scope);

    Ok(())
}
