use crate::block::prelude::*;
use crate::inline::{try_into_inlines, Inline};
use smallvec::{Array, SmallVec};
use PeekState::*;

pub(super) fn peek_quote_author<'e>(
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
) -> Option<impl IntoIterator<Item = Inline<'e>>> {
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

enum PeekState {
    Initial,
    List,
    Item,
    ItemEnd,
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
