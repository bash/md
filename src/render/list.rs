use super::prelude::*;
use crate::block::{render_block_from_event, try_render_block_from_event, Block};
use crate::inline::{into_inlines, try_into_inlines};
use crate::prefix::Prefix;
use crate::style::StyledStr;
use fmtastic::BallotBox;

// TODO: refactor

pub(crate) struct List {
    pub(crate) first_item_number: Option<u64>,
}

impl Block for List {
    fn kind(&self) -> BlockKind {
        BlockKind::List
    }

    fn render<'e>(
        self,
        events: Events<'_, 'e, '_>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut Writer,
    ) -> io::Result<()> {
        let mut list_type = list_style_type(self.first_item_number, ctx);

        terminated_for! {
            for event in terminated!(events, Event::End(TagEnd::List(..))) {
                reachable! {
                    let Event::Start(Tag::Item) = event {
                        item(list_type.clone(), events, ctx, w)?;
                        list_type.increment();
                    }
                }
            }
        }

        Ok(())
    }
}

fn list_style_type(first_item_number: Option<u64>, ctx: &Context) -> ListStyleType {
    match first_item_number {
        Some(n) => ListStyleType::Numbered(n),
        None => ListStyleType::Bulleted(ctx.bullet().to_owned()),
    }
}

fn prefix(list_type: &ListStyleType) -> Prefix {
    let first = match list_type {
        ListStyleType::Numbered(n) => format!("{n}. "),
        ListStyleType::Bulleted(bullet) => format!("{bullet} "),
        ListStyleType::TaskList(checked) => format!("{:#} ", BallotBox(*checked)),
    };
    let style = match list_type {
        ListStyleType::Bulleted(_) | ListStyleType::Numbered(_) => Style::new().bold(),
        ListStyleType::TaskList(_) => Style::new(),
    };
    Prefix::continued(StyledStr::new(first, style))
}

fn list_style_type_from_item(events: Events) -> Option<ListStyleType> {
    let mut events = events.lookahead();
    if let Event::TaskListMarker(checked) = events.next()? {
        _ = events.commit();
        Some(ListStyleType::TaskList(checked))
    } else {
        None
    }
}

fn item<'e>(
    list_type: ListStyleType,
    events: Events<'_, 'e, '_>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut Writer,
) -> io::Result<()> {
    let list_type = list_style_type_from_item(events).unwrap_or(list_type);
    let ctx = ctx.block(prefix(&list_type)).list_depth_incremented();
    let mut list_state = Some(ListItemState::Inlines(None));

    while let Some(s) = list_state.take() {
        list_state = match s {
            ListItemState::Inlines(event) => list_item_inlines(event, events, &ctx, w)?,
            ListItemState::Blocks(event) => list_item_blocks(event, events, &ctx, w)?,
        };
    }

    Ok(())
}

fn list_item_inlines<'e>(
    first_event: Option<Event<'e>>,
    events: Events<'_, 'e, '_>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut Writer,
) -> io::Result<Option<ListItemState<'e>>> {
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
                    return Ok(Some(ListItemState::Blocks(rejected_event)));
                }
            }
        }
    }

    writer.end()?;
    Ok(None)
}

fn list_item_blocks<'e>(
    first_event: Event<'e>,
    events: Events<'_, 'e, '_>,
    ctx: &Context<'_, 'e, '_>,
    w: &mut Writer,
) -> io::Result<Option<ListItemState<'e>>> {
    render_block_from_event(first_event, events, ctx, w)?;

    terminated_for! {
        for event in terminated!(events, Event::End(TagEnd::Item)) {
            if let Some(rejected_event) = try_render_block_from_event(event, events, ctx, w)? {
                return Ok(Some(ListItemState::Inlines(Some(rejected_event))))
            }
        }
    }

    Ok(None)
}

enum ListItemState<'a> {
    Inlines(Option<Event<'a>>),
    Blocks(Event<'a>),
}

#[derive(Clone)]
enum ListStyleType {
    Numbered(u64),
    Bulleted(String),
    TaskList(bool),
}

impl ListStyleType {
    fn increment(&mut self) {
        if let ListStyleType::Numbered(n) = self {
            *n += 1;
        }
    }
}
