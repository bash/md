use super::context::{BlockContext, BlockKind};
use super::{prelude::*, wrap_events, BlockRenderer};
use crate::chars::NO_BREAK_SPACE;
use crate::prefix::Prefix;
use crate::render::block;
use crate::style::StyledStr;
use crate::FootnoteDefinitionPlacement::*;
use fmtastic::Superscript;
use pulldown_cmark::CowStr;

pub(super) struct FootnoteDef<'a> {
    pub(super) reference: CowStr<'a>,
}

impl BlockRenderer for FootnoteDef<'_> {
    fn kind(&self) -> BlockKind {
        BlockKind::BlockQuote
    }

    // Yes this is a quite bad implementation, but footnotes are *soooo* annoying:
    // https://github.com/pulldown-cmark/pulldown-cmark/blob/8713a415b04cdb0b7980a9a17c0ed0df0b36395e/pulldown-cmark/specs/footnotes.txt
    //
    // I think it would be quite nice to give the users a choice about where to position footnotes:
    // * in place (current implementation)
    // * end of section (I really like this idea)
    // * end of document (this is how GitHub renders footnotes)
    fn render<'e>(
        self,
        events: Events<'_, 'e, '_>,
        state: &mut State<'e>,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()> {
        // TODO: collapse multiple footnote defs following each other into one logical "section".
        // TODO: write prefix and writeln is getting awfully repetitive...
        if let InPlace = state.options().footnote_definition_placement {
            write_divider(w, b)?;
        }

        // TODO: dimmed only has an effect on dark backgrounds,
        // we should have a solution for light themes too...
        let number = state.footnotes().get_number(&self.reference);
        let b = b.child(prefix(number)).styled(Style::new().dimmed());

        take! {
            for event in events; until Event::End(TagEnd::FootnoteDefinition) => {
                match state.options().footnote_definition_placement {
                    EndOfDocument => state.footnotes().push(&self.reference, event),
                    InPlace => block(event, events, state, w, &b)?,
                }
            }
        }
        Ok(())
    }
}

pub(super) fn render_collected_footnotes<'e>(
    state: &mut State<'e>,
    w: &mut Writer,
    b: &BlockContext,
) -> io::Result<()> {
    write_divider(w, b)?;

    for footnote in state.footnotes().take() {
        let mut events = footnote.events.into_iter();
        let mut events = wrap_events(&mut events);
        while let Some(event) = events.next() {
            let b = b
                .child(prefix(footnote.number))
                .styled(Style::new().dimmed());
            block(event, &mut events, state, w, &b)?
        }
    }

    Ok(())
}

fn write_divider(w: &mut Writer, b: &BlockContext) -> io::Result<()> {
    w.write_prefix(b)?;
    writeln!(w, "──────")
}

fn prefix(number: usize) -> Prefix {
    Prefix::continued(StyledStr::new(
        format!("{}{NO_BREAK_SPACE}", Superscript(number)),
        Style::new().bold(),
    ))
}
