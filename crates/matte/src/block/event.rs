use crate::block::render_block as render;
use crate::block_quote::BlockQuote;
use crate::context::Context;
use crate::heading::Heading;
use crate::list::List;
use crate::{CodeBlock, Events, FootnoteDef, Paragraph, Rule, Table};
use pulldown_cmark::{Event, Tag, TagEnd};
use std::io;

pub(crate) fn render_block_from_event<'e>(
    event: Event<'e>,
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut dyn io::Write,
) -> io::Result<()> {
    if let Some(rejected) = try_render_block_from_event(event, events, ctx, w)? {
        panic!("Unexpected event {:?} in block context", rejected);
    }
    Ok(())
}

pub(crate) fn try_render_block_from_event<'e>(
    event: Event<'e>,
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut dyn io::Write,
) -> io::Result<Option<Event<'e>>> {
    use Event::Start;
    match event {
        Start(Tag::Paragraph) => render(Paragraph, events, ctx, w)?,
        Start(Tag::Heading { level, .. }) => render(Heading { level }, events, ctx, w)?,
        Start(Tag::BlockQuote(kind)) => render(BlockQuote { kind }, events, ctx, w)?,
        Start(Tag::CodeBlock(kind)) => render(CodeBlock { kind }, events, ctx, w)?,
        Start(Tag::HtmlBlock) => html_block(events)?,
        Start(Tag::List(first_item_number)) => render(List { first_item_number }, events, ctx, w)?,
        Start(Tag::FootnoteDefinition(reference)) => {
            render(FootnoteDef { reference }, events, ctx, w)?
        }
        Start(Tag::Table(alignments)) => render(Table { alignments }, events, ctx, w)?,
        Start(Tag::MetadataBlock(_)) => metadata_block(events)?,
        Event::Rule => render(Rule, events, ctx, w)?,
        event => return Ok(Some(event)),
    }

    Ok(None)
}

fn metadata_block<'e>(events: &mut impl Events<'e>) -> io::Result<()> {
    terminated!(events, Event::End(TagEnd::MetadataBlock(..))).for_each(|_event| {});
    Ok(())
}

fn html_block<'e>(events: &mut impl Events<'e>) -> io::Result<()> {
    terminated!(events, Event::End(TagEnd::HtmlBlock)).for_each(|_event| {});
    Ok(())
}
