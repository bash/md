use self::prelude::*;
use crate::block::render_block_from_event;
use crate::context::State;
use crate::lookahead::Lookaheadable;
use crate::options::Options;

mod code_block;
mod footnote_def;
mod paragraph;
mod rule;
mod table;

pub(crate) use code_block::*;
pub(crate) use footnote_def::*;
pub(crate) use paragraph::*;
pub(crate) use rule::*;
pub(crate) use table::*;

// TODO: get rid of this
mod prelude {
    pub(super) use super::Events;
    pub(super) use crate::block::BlockKind;
    pub(super) use crate::context::Context;
    pub(super) use crate::writer::Writer;
    pub(super) use anstyle::{Reset, Style};
    pub(super) use pulldown_cmark::{Event, TagEnd};
    pub(super) use std::io;
    pub(super) use std::io::Write as _;
}

// TODO: these lifetimes are horrible, make them clearer
pub(crate) type EventsOwned<'b, 'c> =
    Lookaheadable<Event<'b>, &'c mut dyn Iterator<Item = Event<'b>>>;
pub(crate) type Events<'a, 'b, 'c> = &'a mut EventsOwned<'b, 'c>;

pub fn render<'e, I, W>(mut events: I, mut output: W, options: Options) -> io::Result<()>
where
    I: Iterator<Item = Event<'e>>,
    W: io::Write,
{
    let mut events = wrap_events(&mut events);
    let state = State::new(options);
    let ctx = Context::new(&state);
    let mut writer = Writer::new(&mut output);

    while let Some(event) = events.next() {
        render_block_from_event(event, &mut events, &ctx, &mut writer)?;
    }

    render_collected_footnotes(&ctx, &mut writer)?;

    Ok(())
}

/// Parser options supported by [`render`].
/// All of these are enabled by default when running `matte`.
pub const fn supported_parser_options() -> pulldown_cmark::Options {
    use pulldown_cmark::Options;
    Options::ENABLE_FOOTNOTES
        .union(Options::ENABLE_TASKLISTS)
        .union(Options::ENABLE_TABLES)
        .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
        .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
        .union(Options::ENABLE_STRIKETHROUGH)
        .union(Options::ENABLE_MATH)
        .union(Options::ENABLE_GFM) // Enables admonitions i.e. [!NOTE], ...
}

pub(crate) fn wrap_events<'b, 'c>(
    events: &'c mut dyn Iterator<Item = Event<'b>>,
) -> EventsOwned<'b, 'c> {
    Lookaheadable::new(events)
}
