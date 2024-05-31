use self::prelude::*;
use crate::lookahead::Lookaheadable;
use crate::options::Options;

#[macro_use]
mod macros;

mod block_quote;
mod code_block;
mod context;
mod footnote_def;
mod heading;
mod inline;
mod list;
mod paragraph;
mod rule;
mod state;
mod table;
mod writer;

use block_quote::*;
use code_block::*;
use context::{BlockContext, BlockKind};
use footnote_def::*;
use heading::*;
use list::*;
use paragraph::*;
use rule::*;
use table::*;

mod prelude {
    pub(super) use super::state::State;
    pub(super) use super::writer::Writer;
    pub(super) use super::Events;
    pub(super) use anstyle::{Reset, Style};
    pub(super) use pulldown_cmark::{Event, Tag, TagEnd};
    pub(super) use std::io;
    pub(super) use std::io::Write as _;
}

// TODO: these lifetimes are horrible, make them clearer
type EventsOwned<'b, 'c> = Lookaheadable<Event<'b>, &'c mut dyn Iterator<Item = Event<'b>>>;
type Events<'a, 'b, 'c> = &'a mut EventsOwned<'b, 'c>;

pub fn render<'e, I, W>(mut events: I, mut output: W, options: Options) -> io::Result<()>
where
    I: Iterator<Item = Event<'e>>,
    W: io::Write,
{
    let mut events = wrap_events(&mut events);
    let mut state = State::new(options);
    let mut writer = Writer::new(&mut output);
    let block_ctx = BlockContext::default();

    while let Some(event) = events.next() {
        block(event, &mut events, &mut state, &mut writer, &block_ctx)?;
    }

    Ok(())
}

pub fn default_parser_options() -> pulldown_cmark::Options {
    use pulldown_cmark::Options;
    Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_MATH
        | Options::ENABLE_GFM // Enables admonitions i.e. [!NOTE], ...
}

fn wrap_events<'b, 'c>(events: &'c mut dyn Iterator<Item = Event<'b>>) -> EventsOwned<'b, 'c> {
    Lookaheadable::new(events)
}

fn block(
    event: Event,
    events: Events,
    state: &mut State,
    w: &mut Writer,
    b: &BlockContext,
) -> io::Result<()> {
    if let Some(rejected) = try_block(event, events, state, w, b)? {
        panic!("Unexpected event {:?} in block context", rejected);
    }
    Ok(())
}

trait BlockRenderer {
    fn looks_for_end_tag(&self) -> bool {
        true
    }

    fn kind(&self) -> BlockKind;

    fn render(
        self,
        events: Events,
        state: &mut State,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()>;
}

fn try_block<'a>(
    event: Event<'a>,
    events: Events,
    state: &mut State,
    w: &mut Writer,
    b: &BlockContext,
) -> io::Result<Option<Event<'a>>> {
    match event {
        Event::Start(Tag::Paragraph) => render_block(Paragraph, events, state, w, b)?,
        Event::Start(Tag::Heading { level, .. }) => {
            render_block(Heading { level }, events, state, w, b)?
        }
        Event::Start(Tag::BlockQuote(kind)) => {
            render_block(BlockQuote { kind }, events, state, w, b)?
        }
        Event::Start(Tag::CodeBlock(kind)) => {
            render_block(CodeBlock { kind }, events, state, w, b)?
        }
        Event::Start(Tag::HtmlBlock) => html_block(events)?,
        Event::Start(Tag::List(first_item_number)) => {
            render_block(List { first_item_number }, events, state, w, b)?
        }
        Event::Start(Tag::FootnoteDefinition(reference)) => {
            render_block(FootnoteDef { reference }, events, state, w, b)?
        }
        Event::Start(Tag::Table(alignments)) => {
            render_block(Table { alignments }, events, state, w, b)?
        }
        Event::Start(Tag::MetadataBlock(_)) => metadata_block(events)?,
        Event::Rule => render_block(Rule, events, state, w, b)?,
        event => return Ok(Some(event)),
    }

    Ok(None)
}

fn render_block<H: BlockRenderer>(
    handler: H,
    events: Events,
    state: &mut State,
    w: &mut Writer,
    b: &BlockContext,
) -> io::Result<()> {
    if !is_blank(&handler, events) {
        w.write_block_start(b)?;
    }
    let kind = handler.kind();
    b.set_current_block(kind);
    handler.render(events, state, w, b)?;
    b.set_previous_block(kind);
    Ok(())
}

fn is_blank(handler: &impl BlockRenderer, events: Events) -> bool {
    let mut events = events.lookahead();
    handler.looks_for_end_tag() && matches!(events.next(), None | Some(Event::End(_)))
}

fn metadata_block(events: Events) -> io::Result<()> {
    take! {
        for _event in events; until Event::End(TagEnd::MetadataBlock(..)) => {}
    }
    Ok(())
}

fn html_block(events: Events) -> io::Result<()> {
    take! {
        for _event in events; until Event::End(TagEnd::HtmlBlock) => {}
    }
    Ok(())
}
