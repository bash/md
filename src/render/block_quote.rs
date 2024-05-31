use self::classification::{classify, Kind};
use super::context::BlockContext;
use super::{block, State};
use super::{prelude::*, BlockRenderer};
use crate::inline::try_into_inlines;
use crate::inline::Inline;
use crate::prefix::Prefix;
use crate::render::context::BlockKind;
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
        state: &mut State<'e>,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()> {
        let kind = classify(events, self.kind);
        let b = b.child(prefix(kind));
        write_title(kind, state, w, &b)?;

        terminated_for! {
            for event in terminated!(events, Event::End(TagEnd::BlockQuote)) {
                block(event, events, state, w, &b)?;
            }
        }

        if let Some(author) = quote_author(events, state) {
            let b = b
                .child(Prefix::continued("  ― "))
                .styled(Style::new().italic());
            w.inline_writer(state, &b).write_all(author)?;
        }

        b.set_previous_block(BlockKind::BlockQuote);

        Ok(())
    }
}

fn write_title(
    kind: Option<Kind>,
    state: &mut State,
    w: &mut Writer,
    b: &BlockContext,
) -> io::Result<()> {
    if let Some(kind) = kind {
        if let Some(title) = kind.title(state.options().symbol_repertoire) {
            w.write_prefix(b)?;
            writeln!(w, "{}{title}{Reset}", kind.style().bold())?;
        }
    }
    Ok(())
}

fn prefix(kind: Option<Kind>) -> Prefix {
    let style = kind.map(|k| k.style()).unwrap_or_default();
    Prefix::uniform(format!("{style}┃{Reset} "))
}

fn quote_author<'a>(
    events: Events<'_, 'a, '_>,
    s: &mut State,
) -> Option<impl IntoIterator<Item = Inline<'a>>> {
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
                if try_push_inlines(&mut inlines, event, s) {
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

fn try_push_inlines<'a, A: Array<Item = Inline<'a>>>(
    buf: &mut SmallVec<A>,
    event: Event<'a>,
    state: &mut State,
) -> bool {
    try_into_inlines(event, state)
        .map(|inline| buf.extend(inline))
        .is_ok()
}
