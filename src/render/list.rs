use crate::fragment::Fragments;
use crate::prefix::Prefix;
use crate::render::block;
use crate::render::fragment::FragmentsExt;

use super::{Events, State};
use anstyle::Reset;
use anstyle::Style;
use fmtastic::BallotBox;
use pulldown_cmark::{Event, Tag, TagEnd};
use std::io;
use std::mem;
use textwrap::core::display_width;

// TODO: indentation for wrapped lines
// TODO: counters for nested lists
// TODO: customizable list bullet
// TODO: omit bullet point for lists with checkboxes

pub(super) fn list(
    first_item_number: Option<u64>,
    mut events: Events,
    state: &mut State,
) -> io::Result<()> {
    state.write_block_start()?;

    let mut list_type = list_style_type(first_item_number, state);
    take! {
        for event in events; until Event::End(TagEnd::List(..)) => {
            if let Event::Start(Tag::Item) = event {
                item(list_type.clone(), &mut events, state)?;
                list_type.increment();
            } else {
                unreachable!();
            }
        }
    }

    Ok(())
}

fn list_style_type(first_item_number: Option<u64>, state: &mut State) -> ListStyleType {
    match first_item_number {
        Some(n) => ListStyleType::Numbered(n),
        None => ListStyleType::Bulleted(state.bullet().to_owned()),
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
    Prefix::uniform(" ".repeat(display_width(&first)))
        .with_first_special(format!("{style}{first}{Reset}"))
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

fn item<'a>(list_type: ListStyleType, events: Events, state: &mut State) -> io::Result<()> {
    let list_type = list_style_type_from_item(events).unwrap_or(list_type);
    state.block(
        |b| b.prefix(prefix(&list_type)).list(),
        |state| {
            let mut fragments = Fragments::default();

            take! {
                for event in events; until Event::End(TagEnd::Item) => {
                    if !fragments.try_push_event(&event, state) {
                        // If we get a block event, we only get block events from here on out.
                        state.write_fragments(mem::take(&mut fragments))?;
                        block(event, events, state)?;
                    }
                }
            }

            state.write_fragments(fragments)
        },
    )
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
