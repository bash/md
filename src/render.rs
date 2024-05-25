use crate::options::Options;
use pulldown_cmark::{Event, Tag, TagEnd, TextMergeStream};
use std::io::{self};

#[macro_use]
mod macros;

mod fragment;
mod state;
use state::{Block, State};

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

type Events<'a, 'b> = &'a mut dyn Iterator<Item = Event<'b>>;

pub fn render<'a, I, W>(input: &mut I, output: &mut W, options: Options) -> io::Result<()>
where
    I: Iterator<Item = Event<'a>>,
    W: io::Write,
{
    let mut events = TextMergeStream::new(input);
    let mut state = State::new(output, options);

    while let Some(event) = events.next() {
        block(event, &mut events, &mut state)?;
    }

    Ok(())
}

fn block(event: Event, events: Events, state: &mut State) -> io::Result<()> {
    match event {
        Event::Start(Tag::Paragraph) => paragraph(events, state),
        Event::Start(Tag::Heading { level, .. }) => heading(level, events, state),
        Event::Start(Tag::BlockQuote(_kind)) => block_quote(events, state),
        Event::Start(Tag::CodeBlock(kind)) => code_block(kind, events, state),
        Event::Start(Tag::HtmlBlock) => html_block(events),
        Event::Start(Tag::List(first_item_number)) => list(first_item_number, events, state),
        Event::Start(Tag::FootnoteDefinition(reference)) => footnote_def(&reference, events, state),
        Event::Start(Tag::Table(alignment)) => table(alignment, events, state),
        Event::Start(Tag::MetadataBlock(_)) => metadata_block(events),
        Event::Rule => rule(state),
        _ => unreachable!("you have found a bug!"),
    }
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
