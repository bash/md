use super::prelude::*;
use anstyle::AnsiColor::Red;
use pulldown_cmark::Alignment;

pub(super) fn table(
    _alignment: Vec<Alignment>,
    events: Events,
    _state: &mut State,
    w: &mut Writer,
) -> io::Result<()> {
    w.write_block_start()?;

    w.write_prefix()?;
    writeln!(w, "{}[TODO: table]{}", Red.on_default().invert(), Reset)?;

    take! { for event in events; until Event::End(TagEnd::Table) => { } }

    Ok(())
}
