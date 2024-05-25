use super::{Events, State};
use crate::render::block;
use pulldown_cmark::{Event, TagEnd};
use std::io;

// Yes this is a quite bad implementation, but footnotes are *soooo* annoying:
// https://github.com/pulldown-cmark/pulldown-cmark/blob/8713a415b04cdb0b7980a9a17c0ed0df0b36395e/pulldown-cmark/specs/footnotes.txt
pub(super) fn footnote_def(_reference: &str, events: Events, state: &mut State) -> io::Result<()> {
    take! {
        // TODO: numbering
        for event in events; until Event::End(TagEnd::FootnoteDefinition) => {
            block(event, events, state)?;
        }
    }
    Ok(())
}
