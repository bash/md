use super::{block, Block, Events, State};
use pulldown_cmark::{Event, TagEnd};
use std::io;

pub(super) fn block_quote(events: Events, state: &mut State) -> io::Result<()> {
    state.write_block_start()?;
    state.block(Block::default().with_prefix("┃ "), |state| {
        take! {
            for event in events; until Event::End(TagEnd::BlockQuote) => {
                block(event, events, state)?;
            }
        }
        Ok(())
    })
}
