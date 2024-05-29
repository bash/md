use self::classification::{classify, Kind};
use super::{block, Events, State};
use crate::fragment::Fragment;
use crate::prefix::Prefix;
use anstyle::{Reset, Style};
use pulldown_cmark::{BlockQuoteKind, Event, Tag, TagEnd};
use std::io;

mod classification;

pub(super) fn block_quote(
    kind: Option<BlockQuoteKind>,
    events: Events,
    state: &mut State,
) -> io::Result<()> {
    state.write_block_start()?;

    let kind = classify(events, kind);

    state.block::<io::Result<_>>(
        |b| b.prefix(prefix(kind)),
        |state| {
            write_title(kind, state)?;

            take! {
                for event in events; until Event::End(TagEnd::BlockQuote) => {
                    block(event, events, state)?;
                }
            }
            Ok(())
        },
    )?;
    // if let Some(author) = quote_author(events, state) {
    //     state.write_blank_line()?;
    //     let prefix = Prefix::uniform("    ").with_first_special("  ― ");
    //     state.block::<io::Result<_>>(
    //         |b| b.prefix(prefix).styled(|s| s.italic()),
    //         |state| state.fragment_writer().write_all(author),
    //     )?;
    // }
    Ok(())
}

fn write_title(kind: Option<Kind>, state: &mut State) -> io::Result<()> {
    if let Some(kind) = kind {
        if let Some(title) = kind.title(state.options().symbol_repertoire) {
            state.write_prefix()?;
            writeln!(state.writer(), "{}{title}{Reset}", kind.style().bold())?;
        }
    }
    Ok(())
}

fn prefix(kind: Option<Kind>) -> Prefix {
    let style = kind.map(|k| k.style()).unwrap_or(Style::new());
    Prefix::uniform(format!("{style}┃{Reset} "))
}

// fn quote_author<'a>(
//     events: Events<'_, 'a, '_>,
//     render_state: &mut State,
// ) -> Option<Vec<Fragment<'a>>> {
//     enum PeekState {
//         Initial,
//         List,
//         Item,
//         ItemEnd,
//     }

//     use PeekState::*;
//     let mut state = Initial;
//     let mut events = events.lookahead();
//     let mut fragments = Vec::default();
//     while let Some(event) = events.next() {
//         state = match (state, &event) {
//             (Initial, Event::Start(Tag::List(None))) => List,
//             (List, Event::Start(Tag::Item)) => Item,
//             (Item, event) if fragments.try_push_event(&event, render_state) => Item,
//             (Item, Event::End(TagEnd::Item)) => ItemEnd,
//             (ItemEnd, Event::End(TagEnd::List(_))) => {
//                 _ = events.commit();
//                 return Some(fragments);
//             }
//             _unexpected => return None,
//         };
//     }
//     None
// }
