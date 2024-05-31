use self::prelude::*;
use crate::context::{BlockKind, State};
use crate::lookahead::Lookaheadable;
use crate::options::Options;

mod block_quote;
mod code_block;
mod footnote_def;
mod heading;
mod list;
mod paragraph;
mod rule;
mod table;
mod writer;

use block_quote::*;
use code_block::*;
use footnote_def::*;
use heading::*;
use list::*;
use paragraph::*;
use rule::*;
use table::*;

mod prelude {
    pub(super) use super::writer::Writer;
    pub(super) use super::Events;
    pub(super) use crate::context::{BlockKind, Context};
    pub(super) use anstyle::{Reset, Style};
    pub(super) use pulldown_cmark::{Event, Tag, TagEnd};
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
        block(event, &mut events, &ctx, &mut writer)?;
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

fn block<'e>(
    event: Event<'e>,
    events: Events<'_, 'e, '_>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut Writer,
) -> io::Result<()> {
    if let Some(rejected) = try_block(event, events, ctx, w)? {
        panic!("Unexpected event {:?} in block context", rejected);
    }
    Ok(())
}

trait BlockRenderer {
    fn looks_for_end_tag(&self) -> bool {
        true
    }

    fn is_blank(&self, _ctx: &Context) -> bool {
        false
    }

    fn kind(&self) -> BlockKind;

    fn render<'e>(
        self,
        events: Events<'_, 'e, '_>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut Writer,
    ) -> io::Result<()>;
}

fn try_block<'e>(
    event: Event<'e>,
    events: Events<'_, 'e, '_>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut Writer,
) -> io::Result<Option<Event<'e>>> {
    match event {
        Event::Start(Tag::Paragraph) => render_block(Paragraph, events, ctx, w)?,
        Event::Start(Tag::Heading { level, .. }) => {
            render_block(Heading { level }, events, ctx, w)?
        }
        Event::Start(Tag::BlockQuote(kind)) => render_block(BlockQuote { kind }, events, ctx, w)?,
        Event::Start(Tag::CodeBlock(kind)) => render_block(CodeBlock { kind }, events, ctx, w)?,
        Event::Start(Tag::HtmlBlock) => html_block(events)?,
        Event::Start(Tag::List(first_item_number)) => {
            render_block(List { first_item_number }, events, ctx, w)?
        }
        Event::Start(Tag::FootnoteDefinition(reference)) => {
            render_block(FootnoteDef { reference }, events, ctx, w)?
        }
        Event::Start(Tag::Table(alignments)) => render_block(Table { alignments }, events, ctx, w)?,
        Event::Start(Tag::MetadataBlock(_)) => metadata_block(events)?,
        Event::Rule => render_block(Rule, events, ctx, w)?,
        event => return Ok(Some(event)),
    }

    Ok(None)
}

fn render_block<'e, H: BlockRenderer>(
    handler: H,
    events: Events<'_, 'e, '_>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut Writer,
) -> io::Result<()> {
    if !is_blank(&handler, events, ctx) {
        w.write_block_start(ctx)?;
    }
    let kind = handler.kind();
    ctx.set_current_block(kind);
    handler.render(events, ctx, w)?;
    ctx.set_previous_block(kind);
    Ok(())
}

fn is_blank(handler: &impl BlockRenderer, events: Events, ctx: &Context) -> bool {
    let mut events = events.lookahead();
    handler.is_blank(ctx)
        || (handler.looks_for_end_tag() && matches!(events.next(), None | Some(Event::End(_))))
}

fn metadata_block(events: Events) -> io::Result<()> {
    terminated!(events, Event::End(TagEnd::MetadataBlock(..))).for_each(|_event| {});
    Ok(())
}

fn html_block(events: Events) -> io::Result<()> {
    terminated!(events, Event::End(TagEnd::HtmlBlock)).for_each(|_event| {});
    Ok(())
}
