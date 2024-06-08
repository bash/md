use super::counter_style::CounterStyle;
use super::task_list::TaskListMarker;
use crate::block::prelude::*;
use crate::block::{render_block_from_event, try_render_block_from_event};
use crate::inline::{into_inlines, try_into_inlines};
use crate::prefix::Prefix;
use anstyle::Style;

pub(super) fn render_item<'e>(
    counter: &CounterStyle,
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut dyn Write,
) -> io::Result<()> {
    let ctx = ctx
        .block(item_prefix(counter, events), Style::default())
        .list_depth_incremented();
    render_item_contents(events, &ctx, w)
}

enum ListItemState<'e> {
    Inlines(Option<Event<'e>>),
    Blocks(Event<'e>),
    Complete,
}

fn render_item_contents<'e>(
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut dyn Write,
) -> io::Result<()> {
    let mut state = ListItemState::Inlines(None);
    loop {
        state = match state {
            ListItemState::Inlines(event) => list_item_inlines(event, events, ctx, w)?,
            ListItemState::Blocks(event) => list_item_blocks(event, events, ctx, w)?,
            ListItemState::Complete => break,
        };
    }
    Ok(())
}

fn item_prefix<'e>(list_type: &CounterStyle, events: &mut impl Events<'e>) -> Prefix {
    match TaskListMarker::try_consume(events) {
        Some(marker) => marker.to_prefix(),
        None => list_type.to_prefix(),
    }
}

fn list_item_inlines<'e>(
    first_event: Option<Event<'e>>,
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
    mut w: &mut dyn Write,
) -> io::Result<ListItemState<'e>> {
    let mut writer = w.inline_writer(ctx);

    if let Some(event) = first_event {
        writer.write_iter(into_inlines(event, ctx))?;
    }

    terminated_for! {
        for event in terminated!(events, Event::End(TagEnd::Item)) {
            match try_into_inlines(event, ctx) {
                Ok(inlines) => writer.write_iter(inlines)?,
                Err(rejected_event) => {
                    writer.end()?;
                    return Ok(ListItemState::Blocks(rejected_event));
                }
            }
        }
    }

    writer.end()?;
    Ok(ListItemState::Complete)
}

fn list_item_blocks<'e>(
    first_event: Event<'e>,
    events: &mut impl Events<'e>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut dyn Write,
) -> io::Result<ListItemState<'e>> {
    render_block_from_event(first_event, events, ctx, w)?;

    terminated_for! {
        for event in terminated!(events, Event::End(TagEnd::Item)) {
            if let Some(rejected_event) = try_render_block_from_event(event, events, ctx, w)? {
                return Ok(ListItemState::Inlines(Some(rejected_event)))
            }
        }
    }

    Ok(ListItemState::Complete)
}
