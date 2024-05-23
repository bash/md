use super::Context;
use crate::render::block;
use pulldown_cmark::{Event, TagEnd};
use std::io;

pub(super) fn block_quote(
    events: &mut dyn Iterator<Item = Event<'_>>,
    ctx: &mut Context,
) -> io::Result<()> {
    ctx.write_block_start()?;

    let mut ctx = ctx.scope(|s| s.with_prefix("┃ "));

    take! {
        for event in events; until Event::End(TagEnd::BlockQuote) => {
            block(event, events, &mut ctx)?;
        }
    }

    Ok(())
}
