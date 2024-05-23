use super::RenderState;
use anstyle::{AnsiColor, Reset};
use pulldown_cmark::{Alignment, Event, TagEnd};
use std::io::{self, Write as _};

pub(super) fn table(
    _alignment: Vec<Alignment>,
    events: &mut dyn Iterator<Item = Event<'_>>,
    state: &mut RenderState,
) -> io::Result<()> {
    state.write_block_start()?;

    writeln!(
        state.writer(),
        "{}[TODO: table]{}",
        AnsiColor::Red.on_default().invert(),
        Reset
    )?;

    take! { for event in events; until Event::End(TagEnd::Table) => { } }

    Ok(())
}
