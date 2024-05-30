use super::prelude::*;
use crate::prefix::Prefix;
use crate::render::block;
use fmtastic::Superscript;

// Yes this is a quite bad implementation, but footnotes are *soooo* annoying:
// https://github.com/pulldown-cmark/pulldown-cmark/blob/8713a415b04cdb0b7980a9a17c0ed0df0b36395e/pulldown-cmark/specs/footnotes.txt
//
// I think it would be quite nice to give the users a choice about where to position footnotes:
// * in place (current implementation)
// * end of section (I really like this idea)
// * end of document (this is how GitHub renders footnotes)
pub(super) fn footnote_def(
    reference: &str,
    events: Events,
    state: &mut State,
    w: &mut Writer,
) -> io::Result<()> {
    let number = state.get_footnote_number(reference);
    let prefix = Prefix::continued(format!(
        "{}{}{Reset} ",
        Style::new().bold(),
        Superscript(number)
    ));

    // TODO: collapse multiple footnote defs following each other into one logical "section".
    w.write_block_start()?;
    // TODO: write prefix and writeln is getting awfully repetitive...
    w.write_prefix()?;
    writeln!(w, "──────")?;

    w.block(
        // TODO: dimmed only has an effect on dark backgrounds,
        // we should have a solution for light themes too...
        |b| b.prefix(prefix).styled(|s| s.dimmed()),
        |w| {
            take! {
                for event in events; until Event::End(TagEnd::FootnoteDefinition) => {
                    block(event, events, state, w)?;
                }
            }
            Ok(())
        },
    )
}
