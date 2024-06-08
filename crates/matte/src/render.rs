use self::prelude::*;
use crate::block::render_block_from_event;
use crate::context::State;
use crate::lookahead::{IteratorWithLookahead, Lookaheadable};
use crate::options::Options;
use trait_set::trait_set;

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
    pub(super) use crate::writer::WriteExt as _;
    pub(super) use anstyle::{Reset, Style};
    pub(super) use pulldown_cmark::{Event, TagEnd};
    pub(super) use std::io::{self, Write};
}

trait_set! {
    pub(crate) trait Events<'e> = IteratorWithLookahead<Item = Event<'e>>;
}

pub fn render<'e, I, W>(events: I, mut output: W, options: Options) -> io::Result<()>
where
    I: Iterator<Item = Event<'e>>,
    W: io::Write,
{
    let mut events = Lookaheadable::new(events);
    let state = State::new(options);
    let ctx = Context::new(&state);

    while let Some(event) = events.next() {
        render_block_from_event(event, &mut events, &ctx, &mut output)?;
    }

    render_collected_footnotes(&ctx, &mut output)?;

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
