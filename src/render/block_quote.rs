use super::{block, Events, State};
use crate::fragment::Fragments;
use crate::prefix::Prefix;
use crate::render::fragment::FragmentsExt;
use anstyle::{AnsiColor, Reset, Style};
use pulldown_cmark::{BlockQuoteKind, Event, Tag, TagEnd};
use std::io;

pub(super) fn block_quote(
    kind: Option<BlockQuoteKind>,
    events: Events,
    state: &mut State,
) -> io::Result<()> {
    state.write_block_start()?;
    state.block::<io::Result<_>>(
        |b| b.prefix(prefix(kind)),
        |state| {
            if let Some(title) = kind.map(title) {
                state.write_prefix()?;
                writeln!(state.writer(), "{}{title}{Reset}", color(kind).bold())?;
                state.unset_first_block();
            }

            take! {
                for event in events; until Event::End(TagEnd::BlockQuote) => {
                    block(event, events, state)?;
                }
            }
            Ok(())
        },
    )?;
    if let Some(author) = quote_author(events, state) {
        state.write_blank_line()?;
        let prefix = Prefix::uniform("    ").with_first_special("  ― ");
        state.block::<io::Result<_>>(
            |b| b.prefix(prefix).styled(|s| s.italic()),
            |state| state.write_fragments(author),
        )?;
    }
    Ok(())
}

fn prefix(kind: Option<BlockQuoteKind>) -> Prefix {
    let style = color(kind);
    Prefix::uniform(format!("{style}┃{Reset} "))
}

fn color(kind: Option<BlockQuoteKind>) -> Style {
    match kind {
        None => Style::new(),
        Some(BlockQuoteKind::Note) => AnsiColor::Blue.on_default(),
        Some(BlockQuoteKind::Tip) => AnsiColor::Green.on_default(),
        Some(BlockQuoteKind::Important) => AnsiColor::Magenta.on_default(),
        Some(BlockQuoteKind::Warning) => AnsiColor::Yellow.on_default(),
        Some(BlockQuoteKind::Caution) => AnsiColor::Red.on_default(),
    }
}

// TODO: make emoji configurable
fn title(kind: BlockQuoteKind) -> &'static str {
    match kind {
        BlockQuoteKind::Note => "ℹ️  Note",
        BlockQuoteKind::Tip => "💡 Tip",
        BlockQuoteKind::Important => "💬 Important",
        BlockQuoteKind::Warning => "⚠️  Warning",
        BlockQuoteKind::Caution => "🛑 Caution",
    }
}

fn quote_author<'a>(events: Events<'_, 'a, '_>, render_state: &mut State) -> Option<Fragments<'a>> {
    use PeekState::*;
    let mut state = Initial;
    let mut events = events.lookahead();
    let mut fragments = Fragments::default();
    while let Some(event) = events.next() {
        state = match (state, &event) {
            (Initial, Event::Start(Tag::List(None))) => List,
            (List, Event::Start(Tag::Item)) => Item,
            (Item, event) if fragments.try_push_event(&event, render_state) => Item,
            (Item, Event::End(TagEnd::Item)) => ItemEnd,
            (ItemEnd, Event::End(TagEnd::List(_))) => {
                _ = events.commit();
                return Some(fragments);
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
