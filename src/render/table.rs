use super::Context;
use anstyle::{AnsiColor, Reset};
use pulldown_cmark::{Alignment, Event, TagEnd};
use std::io::{self, Write as _};

pub(super) fn table(
    _alignment: Vec<Alignment>,
    events: &mut dyn Iterator<Item = Event<'_>>,
    ctx: &mut Context,
) -> io::Result<()> {
    ctx.write_block_start()?;

    writeln!(
        ctx.writer(),
        "{}[TODO: table]{}",
        AnsiColor::Red.on_default().invert(),
        Reset
    )?;

    take! { for event in events; until Event::End(TagEnd::Table) => { } }

    Ok(())
}
