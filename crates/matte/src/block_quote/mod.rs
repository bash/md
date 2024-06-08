use crate::block::prelude::*;
use crate::block::render_block_from_event;
use crate::inline::Inline;
use crate::prefix::Prefix;
use crate::themes::ThemeProvider as _;
use anstyle::{Reset, Style};
use author::peek_quote_author;
use classification::classify;
use pulldown_cmark::BlockQuoteKind;

mod author;
mod classification;
pub(crate) use classification::Kind;

pub(crate) struct BlockQuote {
    pub(crate) kind: Option<BlockQuoteKind>,
}

impl Block for BlockQuote {
    fn kind(&self) -> BlockKind {
        BlockKind::BlockQuote
    }

    fn render<'e>(
        self,
        events: &mut impl Events<'e>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut impl Write,
    ) -> io::Result<()> {
        write_block_quote(self.kind, events, ctx, w)?;

        if let Some(inlines) = peek_quote_author(events, ctx) {
            write_author(inlines, ctx, w)?;
        }

        Ok(())
    }
}

fn write_block_quote<'e>(
    kind: Option<BlockQuoteKind>,
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut impl Write,
) -> io::Result<()> {
    let kind = classify(events, kind);
    let prefix = ctx.theme().block_quote_prefix(kind, ctx);
    let style = ctx.theme().block_quote_style(kind, ctx);
    let ctx = ctx.block(prefix, style);

    write_title(kind, &ctx, w)?;

    terminated_for! {
        for event in terminated!(events, Event::End(TagEnd::BlockQuote)) {
            render_block_from_event(event, events, &ctx, w)?;
        }
    }

    Ok(())
}

fn write_title(kind: Option<Kind>, ctx: &Context<'_, '_, '_>, w: &mut impl Write) -> io::Result<()> {
    if let Some(kind) = kind {
        if let Some(title) = kind.title(ctx.options().symbol_repertoire) {
            w.write_prefix(ctx)?;
            writeln!(w, "{}{title}{Reset}", kind.style().bold())?;
        }
    }
    Ok(())
}

fn write_author<'a>(
    inlines: impl IntoIterator<Item = Inline<'a>>,
    ctx: &Context<'_, '_, '_>,
    w: &mut impl Write,
) -> io::Result<()> {
    // This is not a regular dash, it's a "quotation dash".
    // https://english.stackexchange.com/a/59320
    // It's also how wikipedia displays block quotes with
    // an author: https://en.wikipedia.org/wiki/Template:Blockquote
    let ctx = ctx.block(Prefix::continued("    ― "), Style::new());
    w.inline_writer(&ctx).write_all(inlines)
}
