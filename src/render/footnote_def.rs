use super::context::{BlockContext, BlockKind};
use super::{prelude::*, BlockRenderer};
use crate::chars::NO_BREAK_SPACE;
use crate::prefix::Prefix;
use crate::render::block;
use crate::style::StyledStr;
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
    fn render(
        self,
        events: Events,
        state: &mut State,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()> {
        let number = state.get_footnote_number(&self.reference);
        let prefix = Prefix::continued(StyledStr::new(
            format!("{}{NO_BREAK_SPACE}", Superscript(number)),
            Style::new().bold(),
        ));

        // TODO: collapse multiple footnote defs following each other into one logical "section".
        // TODO: write prefix and writeln is getting awfully repetitive...
        w.write_prefix(b)?;
        writeln!(w, "──────")?;

        // TODO: dimmed only has an effect on dark backgrounds,
        // we should have a solution for light themes too...
        let b = b.child(prefix).styled(Style::new().dimmed());

        take! {
            for event in events; until Event::End(TagEnd::FootnoteDefinition) => {
                block(event, events, state, w, &b)?;
            }
        }
        Ok(())
    }
}
