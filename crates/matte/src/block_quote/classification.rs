use crate::options::SymbolRepertoire;
use crate::render::Events;
use anstyle::{AnsiColor, Style};
use pulldown_cmark::{BlockQuoteKind, Event, Tag};
use AnsiColor::*;
use BlockQuoteKind::*;
use Kind::*;

pub(super) fn classify(events: Events, kind: Option<BlockQuoteKind>) -> Option<Kind> {
    kind.map(Kind::Markup)
        .or_else(|| classify_from_text(events).map(Kind::Text))
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Kind {
    Markup(BlockQuoteKind),
    Text(BlockQuoteKind),
}

impl Kind {
    pub(crate) fn style(self) -> Style {
        match BlockQuoteKind::from(self) {
            Note => Blue.on_default(),
            Tip => Green.on_default(),
            Important => Magenta.on_default(),
            Warning => Yellow.on_default(),
            Caution => Red.on_default(),
        }
    }

    pub(super) fn title(self, symbols: SymbolRepertoire) -> Option<&'static str> {
        // `U+FE0F` is used to request an emoji presentation for an emoji character.
        // has little effect in my tests, hopefully the situation will improve over time.
        match self {
            Markup(Note) if symbols.has_emoji() => Some("ℹ️\u{FE0F} Note"),
            Markup(Note) => Some("Note"),
            Markup(Tip) if symbols.has_emoji() => Some("💡 Tip"),
            Markup(Tip) => Some("Tip"),
            Markup(Important) if symbols.has_emoji() => Some("💬 Important"),
            Markup(Important) => Some("Important"),
            Markup(Warning) if symbols.has_emoji() => Some("⚠️\u{FE0F} Warning"),
            Markup(Warning) => Some("Warning"),
            Markup(Caution) if symbols.has_emoji() => Some("🛑 Caution"),
            Markup(Caution) => Some("Caution"),
            Text(_) => None,
        }
    }
}

impl From<Kind> for BlockQuoteKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Markup(k) => k,
            Kind::Text(k) => k,
        }
    }
}

fn classify_from_text(events: Events) -> Option<BlockQuoteKind> {
    macro_rules! starts_with {
        ($text:ident, $symbol:literal) => {
            $text.trim_start().starts_with($symbol)
        };
    }

    enum PeekState {
        Initial,
        Paragraph,
    }

    use BlockQuoteKind::*;
    use PeekState::*;

    let mut state = Initial;
    let events = events.lookahead();

    for event in events {
        state = match (state, &event) {
            (Initial, Event::Start(Tag::Paragraph)) => Paragraph,
            (Paragraph, Event::Start(Tag::Emphasis | Tag::Strong) | Event::HardBreak) => Paragraph,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{supported_parser_options, wrap_events};
    use pulldown_cmark::Parser;

    #[test]
    fn minimal() {
        for (symbol, kind) in kinds() {
            assert_eq!(Some(*kind), classify(symbol));
        }
    }

    #[test]
    fn leading_whitespace() {
        for (symbol, kind) in kinds() {
            assert_eq!(Some(*kind), classify(&format!("      \t\n \n {symbol}")));
        }
    }

    #[test]
    fn trailing_text() {
        for (symbol, kind) in kinds() {
            assert_eq!(Some(*kind), classify(&format!("{symbol} hello world")));
        }
    }

    #[test]
    fn inside_emphasis() {
        for (symbol, kind) in kinds() {
            assert_eq!(Some(*kind), classify(&format!("_{symbol}_")));
        }
    }

    #[test]
    fn inside_strong() {
        for (symbol, kind) in kinds() {
            assert_eq!(Some(*kind), classify(&format!("**{symbol}**")));
        }
    }

    #[test]
    fn after_hard_break() {
        for (symbol, kind) in kinds() {
            assert_eq!(Some(*kind), classify(&format!("\\\n{symbol}")));
        }
    }

    #[test]
    fn counter_examples() {
        assert_eq!(None, classify("Note"));
        for (symbol, _) in kinds() {
            assert_eq!(None, classify(&format!("- {symbol} List")));
            assert_eq!(None, classify(&format!("~~{symbol}~~")));
            assert_eq!(None, classify(&format!("> {symbol} Nested")));
        }
    }

    fn classify(markdown: &str) -> Option<BlockQuoteKind> {
        let mut parser = Parser::new_ext(markdown, supported_parser_options());
        let mut events = wrap_events(&mut parser);
        classify_from_text(&mut events)
    }

    fn kinds() -> &'static [(&'static str, BlockQuoteKind)] {
        &[
            ("ℹ️", Note),
            ("💡", Tip),
            ("💬", Important),
            ("⚠️", Warning),
            ("🛑", Caution),
        ]
    }
}
