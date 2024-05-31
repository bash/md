use super::prelude::*;
use crate::prefix::Prefix;
use crate::render::block;
use crate::render::inline::{into_inlines, try_into_inlines};
use crate::render::try_block;
use crate::style::StyledStr;
use fmtastic::BallotBox;

// TODO: refactor

pub(super) fn list(
    first_item_number: Option<u64>,
    events: Events,
    state: &mut State,
    w: &mut Writer,
) -> io::Result<()> {
    w.write_block_start()?;

    let mut list_type = list_style_type(first_item_number, state, w);
    take! {
        for event in events; until Event::End(TagEnd::List(..)) => {
            if let Event::Start(Tag::Item) = event {
                item(list_type.clone(), events, state, w)?;
                list_type.increment();
            } else {
                unreachable!();
            }
        }
    }

    Ok(())
}

fn list_style_type(first_item_number: Option<u64>, state: &mut State, w: &Writer) -> ListStyleType {
    match first_item_number {
        Some(n) => ListStyleType::Numbered(n),
        None => ListStyleType::Bulleted(state.bullet(w).to_owned()),
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

fn item(
    list_type: ListStyleType,
    events: Events,
    state: &mut State,
    w: &mut Writer,
) -> io::Result<()> {
    let list_type = list_style_type_from_item(events).unwrap_or(list_type);
    w.block(
        |b| b.prefix(prefix(&list_type)).list(),
        |w| {
            let mut list_state = Some(ListItemState::Inlines(None));

            while let Some(s) = list_state.take() {
                list_state = match s {
                    ListItemState::Inlines(event) => list_item_inlines(event, events, state, w)?,
                    ListItemState::Blocks(event) => list_item_blocks(event, events, state, w)?,
                };
            }

            Ok(())
        },
    )
}

fn list_item_inlines<'a>(
    first_event: Option<Event<'a>>,
    events: Events<'_, 'a, '_>,
    state: &mut State,
    w: &mut Writer,
) -> io::Result<Option<ListItemState<'a>>> {
    let mut writer = w.inline_writer(state);

    if let Some(event) = first_event {
        writer.write_iter(into_inlines(event, state))?;
    }

    take! {
        for event in events; until Event::End(TagEnd::Item) => {
            match try_into_inlines(event, state) {
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

fn list_item_blocks<'a>(
    first_event: Event<'a>,
    events: Events<'_, 'a, '_>,
    state: &mut State,
    w: &mut Writer,
) -> io::Result<Option<ListItemState<'a>>> {
    block(first_event, events, state, w)?;

    take! {
        for event in events; until Event::End(TagEnd::Item) => {
            if let Some(rejected_event) = try_block(event, events, state, w)? {
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
