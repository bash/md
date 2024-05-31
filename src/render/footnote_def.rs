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
        BlockKind::FootnoteDefinition
    }

    fn is_blank(&self, state: &Context) -> bool {
        !matches!(state.options().footnote_definition_placement, InPlace)
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
        ctx: &Context<'_, 'e, '_>,
        w: &mut Writer,
    ) -> io::Result<()> {
        // TODO: collapse multiple footnote defs following each other into one logical "section".
        // TODO: write prefix and writeln is getting awfully repetitive...
        if let InPlace = ctx.options().footnote_definition_placement {
            write_divider(w, ctx)?;
        }

        // TODO: dimmed only has an effect on dark backgrounds,
        // we should have a solution for light themes too...
        let number = ctx.footnotes().get_number(&self.reference);
        let ctx = ctx.block(prefix(number)).styled(Style::new().dimmed());

        terminated_for! {
            for event in terminated!(events, Event::End(TagEnd::FootnoteDefinition)) {
                match ctx.options().footnote_definition_placement {
                    EndOfDocument => ctx.footnotes().push(&self.reference, event),
                    InPlace => block(event, events, &ctx, w)?,
                }
            }
        }
        Ok(())
    }
}

pub(super) fn render_collected_footnotes(ctx: &Context, w: &mut Writer) -> io::Result<()> {
    let footnotes = ctx.footnotes().take();

    if !footnotes.is_empty() {
        w.write_blank_line(ctx)?;
        write_divider(w, ctx)?;

        for footnote in footnotes {
            let mut events = footnote.events.into_iter();
            let mut events = wrap_events(&mut events);
            while let Some(event) = events.next() {
                let ctx = ctx
                    .block(prefix(footnote.number))
                    .styled(Style::new().dimmed());
                block(event, &mut events, &ctx, w)?
            }
        }
    }

    Ok(())
}

fn write_divider(w: &mut Writer, ctx: &Context) -> io::Result<()> {
    w.write_prefix(ctx)?;
    writeln!(w, "──────")
}

fn prefix(number: usize) -> Prefix {
    Prefix::continued(StyledStr::new(
        format!("{}{NO_BREAK_SPACE}", Superscript(number)),
        Style::new().bold(),
    ))
}
