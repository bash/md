use self::classification::{classify, Kind};
use super::prelude::*;
use super::{block, State};
use crate::fragment::Fragment;
use crate::prefix::Prefix;
use crate::render::fragment::try_into_fragments;
use pulldown_cmark::BlockQuoteKind;
use smallvec::{Array, SmallVec};

mod classification;

pub(super) fn block_quote(
    kind: Option<BlockQuoteKind>,
    events: Events,
    state: &mut State,
    w: &mut Writer,
) -> io::Result<()> {
    w.write_block_start()?;

    let kind = classify(events, kind);

    w.block::<io::Result<_>>(
        |b| b.prefix(prefix(kind)),
        |w| {
            write_title(kind, state, w)?;

            take! {
                for event in events; until Event::End(TagEnd::BlockQuote) => {
                    block(event, events, state, w)?;
                }
            }
            Ok(())
        },
    )?;
    if let Some(author) = quote_author(events) {
        let prefix = Prefix::uniform("    ").with_first_special("  ― ");
        w.block::<io::Result<_>>(
            |b| b.prefix(prefix).styled(|s| s.italic()),
            |w| w.fragment_writer(state).write_all(author),
        )?;
    }
    Ok(())
}

fn write_title(kind: Option<Kind>, state: &mut State, w: &mut Writer) -> io::Result<()> {
    if let Some(kind) = kind {
        if let Some(title) = kind.title(state.options().symbol_repertoire) {
            w.write_prefix()?;
            writeln!(w, "{}{title}{Reset}", kind.style().bold())?;
        }
    }
    Ok(())
}

fn prefix(kind: Option<Kind>) -> Prefix {
    let style = kind.map(|k| k.style()).unwrap_or(Style::new());
    Prefix::uniform(format!("{style}┃{Reset} "))
}

fn quote_author<'a>(events: Events<'_, 'a, '_>) -> Option<impl IntoIterator<Item = Fragment<'a>>> {
    enum PeekState {
        Initial,
        List,
        Item,
        ItemEnd,
    }
    use PeekState::*;
    let mut state = Initial;
    let mut events = events.lookahead();
    let mut fragments = SmallVec::<[_; 8]>::default();
    while let Some(event) = events.next() {
        state = match (state, event) {
            (Initial, Event::Start(Tag::List(None))) => List,
            (List, Event::Start(Tag::Item)) => Item,
            (Item, Event::End(TagEnd::Item)) => ItemEnd,
            (Item, event) => {
                if try_push_fragments(&mut fragments, event) {
                    Item
                } else {
                    return None;
                }
            }
            (ItemEnd, Event::End(TagEnd::List(_))) => {
                _ = events.commit();
                return Some(fragments);
            }
            _unexpected => return None,
        };
    }
    None
}

fn try_push_fragments<'a, A: Array<Item = Fragment<'a>>>(
    buf: &mut SmallVec<A>,
    event: Event<'a>,
) -> bool {
    try_into_fragments(event).map(|f| buf.extend(f)).is_ok()
}
