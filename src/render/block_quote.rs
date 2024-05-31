use self::classification::{classify, Kind};
use super::block;
use super::{prelude::*, BlockRenderer};
use crate::inline::try_into_inlines;
use crate::inline::Inline;
use crate::prefix::Prefix;
use pulldown_cmark::BlockQuoteKind;
use smallvec::{Array, SmallVec};

mod classification;

pub(super) struct BlockQuote {
    pub(super) kind: Option<BlockQuoteKind>,
}

impl BlockRenderer for BlockQuote {
    fn kind(&self) -> BlockKind {
        BlockKind::BlockQuote
    }

    fn render<'e>(
        self,
        events: Events<'_, 'e, '_>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut Writer,
    ) -> io::Result<()> {
        let kind = classify(events, self.kind);
        let ctx = ctx.block(prefix(kind));
        write_title(kind, &ctx, w)?;
        terminated_for! {
            for event in terminated!(events, Event::End(TagEnd::BlockQuote)) {
                block(event, events, &ctx, w)?;
            }
        }

        if let Some(author) = quote_author(events, &ctx) {
            let ctx = ctx
                .block(Prefix::continued("  ― "))
                .styled(Style::new().italic());
            w.inline_writer(&ctx).write_all(author)?;
        }

        Ok(())
    }
}

fn write_title(kind: Option<Kind>, ctx: &Context, w: &mut Writer) -> io::Result<()> {
    if let Some(kind) = kind {
        if let Some(title) = kind.title(ctx.options().symbol_repertoire) {
            w.write_prefix(ctx)?;
            writeln!(w, "{}{title}{Reset}", kind.style().bold())?;
        }
    }
    Ok(())
}

fn prefix(kind: Option<Kind>) -> Prefix {
    let style = kind.map(|k| k.style()).unwrap_or_default();
    Prefix::uniform(format!("{style}┃{Reset} "))
}

fn quote_author<'e>(
    events: Events<'_, 'e, '_>,
    ctx: &Context<'_, 'e, '_>,
) -> Option<impl IntoIterator<Item = Inline<'e>>> {
    enum PeekState {
        Initial,
        List,
        Item,
        ItemEnd,
    }
    use PeekState::*;
    let mut state = Initial;
    let mut events = events.lookahead();
    let mut inlines = SmallVec::<[_; 8]>::default();
    while let Some(event) = events.next() {
        state = match (state, event) {
            (Initial, Event::Start(Tag::List(None))) => List,
            (List, Event::Start(Tag::Item)) => Item,
            (Item, Event::End(TagEnd::Item)) => ItemEnd,
            (Item, event) => {
                if try_push_inlines(&mut inlines, event, ctx) {
                    Item
                } else {
                    return None;
                }
            }
            (ItemEnd, Event::End(TagEnd::List(_))) => {
                _ = events.commit();
                return Some(inlines);
            }
            _unexpected => return None,
        };
    }
    None
}

fn try_push_inlines<'e, A: Array<Item = Inline<'e>>>(
    buf: &mut SmallVec<A>,
    event: Event<'e>,
    ctx: &Context<'_, 'e, '_>,
) -> bool {
    try_into_inlines(event, ctx)
        .map(|inline| buf.extend(inline))
        .is_ok()
}
