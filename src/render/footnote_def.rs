use super::{Events, State};
use crate::prefix::Prefix;
use crate::render::block;
use anstyle::{Reset, Style};
use fmtastic::Superscript;
use pulldown_cmark::{Event, TagEnd};
use std::io;

// Yes this is a quite bad implementation, but footnotes are *soooo* annoying:
// https://github.com/pulldown-cmark/pulldown-cmark/blob/8713a415b04cdb0b7980a9a17c0ed0df0b36395e/pulldown-cmark/specs/footnotes.txt
pub(super) fn footnote_def(reference: &str, events: Events, state: &mut State) -> io::Result<()> {
    let number = state.get_footnote_number(reference);
    let prefix = Prefix::continued(format!(
        "{}{}{Reset} ",
        Style::new().bold(),
        Superscript(number)
    ));

    state.write_block_start()?;

    state.write_prefix()?;
    writeln!(state.writer(), "──────")?;

    state.block(
        |b| b.prefix(prefix),
        |state| {
            take! {
                for event in events; until Event::End(TagEnd::FootnoteDefinition) => {
                    block(event, events, state)?;
                }
            }
            Ok(())
        },
    )
}
