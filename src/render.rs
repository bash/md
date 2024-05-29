use crate::lookahead::Lookaheadable;
use crate::options::Options;
use pulldown_cmark::{Event, Tag, TagEnd, TextMergeStream};
use std::io::{self};

#[macro_use]
mod macros;

mod fragment;
mod state;
use state::State;

mod block_quote;
mod code_block;
mod footnote_def;
mod heading;
mod list;
mod paragraph;
mod rule;
mod table;

use block_quote::*;
use code_block::*;
use footnote_def::*;
use heading::*;
use list::*;
use paragraph::*;
use rule::*;
use table::*;

// TODO: these lifetimes are horrible, make them clearer
type EventsOwned<'b, 'c> =
    Lookaheadable<Event<'b>, TextMergeStream<'b, &'c mut dyn Iterator<Item = Event<'b>>>>;
type Events<'a, 'b, 'c> = &'a mut EventsOwned<'b, 'c>;

pub fn render<'a, 'e, W>(
    input: &'a mut dyn Iterator<Item = Event<'e>>,
    output: &mut W,
    options: Options,
) -> io::Result<()>
where
    W: io::Write,
{
    let mut events = wrap_events(input);
    let mut state = State::new(output, options);

    while let Some(event) = events.next() {
        block(event, &mut events, &mut state)?;
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
    Lookaheadable::new(TextMergeStream::new(events))
}

fn block(event: Event, events: Events, state: &mut State) -> io::Result<()> {
    if let Some(rejected) = try_block(event, events, state)? {
        panic!("Unexpected event {:?} in block context", rejected);
    }
    Ok(())
}

fn try_block<'a>(
    event: Event<'a>,
    events: Events,
    state: &mut State,
) -> io::Result<Option<Event<'a>>> {
    match event {
        Event::Start(Tag::Paragraph) => paragraph(events, state)?,
        Event::Start(Tag::Heading { level, .. }) => heading(level, events, state)?,
        Event::Start(Tag::BlockQuote(kind)) => block_quote(kind, events, state)?,
        Event::Start(Tag::CodeBlock(kind)) => code_block(kind, events, state)?,
        Event::Start(Tag::HtmlBlock) => html_block(events)?,
        Event::Start(Tag::List(first_item_number)) => list(first_item_number, events, state)?,
        Event::Start(Tag::FootnoteDefinition(reference)) => {
            footnote_def(&reference, events, state)?
        }
        Event::Start(Tag::Table(alignment)) => table(alignment, events, state)?,
        Event::Start(Tag::MetadataBlock(_)) => metadata_block(events)?,
        Event::Rule => rule(state)?,
        event => return Ok(Some(event)),
    };
    Ok(None)
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
