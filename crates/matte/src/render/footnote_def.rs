use super::prelude::*;
use crate::block::{render_block_from_event, Block};
use crate::chars::NO_BREAK_SPACE;
use crate::lookahead::Lookaheadable;
use crate::prefix::Prefix;
use crate::style::StyledStr;
use crate::FootnoteDefinitionPlacement::*;
use fmtastic::Superscript;
use pulldown_cmark::CowStr;

pub(crate) struct FootnoteDef<'a> {
    pub(crate) reference: CowStr<'a>,
}

impl Block for FootnoteDef<'_> {
    fn kind(&self) -> BlockKind {
        BlockKind::FootnoteDefinition
    }

    fn is_blank(&self, state: &Context<'_, '_, '_>) -> bool {
        !matches!(state.options().footnote_definition_placement, InPlace)
    }

    fn render<'e>(
        self,
        events: &mut impl Events<'e>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut impl Write,
    ) -> io::Result<()> {
        // TODO: collapse multiple footnote defs following each other into one logical "section".
        if let InPlace = ctx.options().footnote_definition_placement {
            write_divider(w, ctx)?;
        }

        let number = ctx.footnotes().get_number(&self.reference);
        let ctx = ctx.block(prefix(number), Style::new().dimmed());

        terminated_for! {
            for event in terminated!(events, Event::End(TagEnd::FootnoteDefinition)) {
                match ctx.options().footnote_definition_placement {
                    EndOfDocument => ctx.footnotes().push(&self.reference, event),
                    InPlace => render_block_from_event(event, events, &ctx, w)?,
                }
            }
        }
        Ok(())
    }
}

pub(super) fn render_collected_footnotes(
    ctx: &Context<'_, '_, '_>,
    w: &mut impl Write,
) -> io::Result<()> {
    let footnotes = ctx.footnotes().take();

    if !footnotes.is_empty() {
        w.write_blank_line(ctx)?;
        write_divider(w, ctx)?;

        for footnote in footnotes {
            let mut events = Lookaheadable::new(footnote.events.into_iter());
            while let Some(event) = events.next() {
                let ctx = ctx.block(prefix(footnote.number), Style::new().dimmed());
                render_block_from_event(event, &mut events, &ctx, w)?
            }
        }
    }

    Ok(())
}

fn write_divider(w: &mut impl Write, ctx: &Context<'_, '_, '_>) -> io::Result<()> {
    w.write_prefix(ctx)?;
    writeln!(w, "──────")
}

fn prefix(number: usize) -> Prefix {
    Prefix::continued(StyledStr::new(
        format!("{}{NO_BREAK_SPACE}", Superscript(number)),
        Style::new().bold(),
    ))
}
