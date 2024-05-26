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

    let kind = kind
        .map(Kind::Gfm)
        .or_else(|| detect_kind_from_text(events).map(Kind::FromText));

    state.block::<io::Result<_>>(
        |b| b.prefix(prefix(kind)),
        |state| {
            if let Some(title) = kind.and_then(title) {
                state.write_prefix()?;
                writeln!(state.writer(), "{}{title}{Reset}", color(kind).bold())?;
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

fn prefix(kind: Option<Kind>) -> Prefix {
    let style = color(kind);
    Prefix::uniform(format!("{style}┃{Reset} "))
}

fn color(kind: Option<Kind>) -> Style {
    match kind.map(BlockQuoteKind::from) {
        None => Style::new(),
        Some(BlockQuoteKind::Note) => AnsiColor::Blue.on_default(),
        Some(BlockQuoteKind::Tip) => AnsiColor::Green.on_default(),
        Some(BlockQuoteKind::Important) => AnsiColor::Magenta.on_default(),
        Some(BlockQuoteKind::Warning) => AnsiColor::Yellow.on_default(),
        Some(BlockQuoteKind::Caution) => AnsiColor::Red.on_default(),
    }
}

// TODO: make emoji configurable
fn title(kind: Kind) -> Option<&'static str> {
    use Kind::Gfm;
    match kind {
        Gfm(BlockQuoteKind::Note) => Some("ℹ️  Note"),
        Gfm(BlockQuoteKind::Tip) => Some("💡 Tip"),
        Gfm(BlockQuoteKind::Important) => Some("💬 Important"),
        Gfm(BlockQuoteKind::Warning) => Some("⚠️  Warning"),
        Gfm(BlockQuoteKind::Caution) => Some("🛑 Caution"),
        _ => None,
    }
}

fn detect_kind_from_text(events: Events) -> Option<BlockQuoteKind> {
    enum PeekState {
        Initial,
        Paragraph,
    }
    use BlockQuoteKind::*;
    use PeekState::*;

    let mut state = Initial;
    let mut events = events.lookahead();

    macro_rules! starts_with {
        ($text:ident, $symbol:literal) => {
            $text.trim_start().starts_with($symbol)
        };
    }

    while let Some(event) = events.next() {
        state = match (state, &event) {
            (Initial, Event::Start(Tag::Paragraph)) => Paragraph,
            (Paragraph, Event::Start(Tag::Emphasis | Tag::Strong)) => Paragraph,
            (Paragraph, Event::End(TagEnd::Emphasis | TagEnd::Strong)) => Paragraph,
            (Paragraph, Event::Text(text)) if starts_with!(text, "ℹ️") => return Some(Note),
            (Paragraph, Event::Text(text)) if starts_with!(text, "💡") => return Some(Tip),
            (Paragraph, Event::Text(text)) if starts_with!(text, "💬") => return Some(Important),
            (Paragraph, Event::Text(text)) if starts_with!(text, "⚠️") => return Some(Warning),
            (Paragraph, Event::Text(text)) if starts_with!(text, "🛑") => return Some(Caution),
            _ => return None,
        }
    }

    None
}

#[derive(Debug, Copy, Clone)]
enum Kind {
    Gfm(BlockQuoteKind),
    FromText(BlockQuoteKind),
}

impl From<Kind> for BlockQuoteKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Gfm(k) => k,
            Kind::FromText(k) => k,
        }
    }
}

fn quote_author<'a>(events: Events<'_, 'a, '_>, render_state: &mut State) -> Option<Fragments<'a>> {
    enum PeekState {
        Initial,
        List,
        Item,
        ItemEnd,
    }

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
