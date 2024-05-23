use crate::fragment::Fragments;
use crate::render::block;
use crate::render::fragment::FragmentsExt;

use super::RenderState;
use pulldown_cmark::{Event, Tag, TagEnd};
use std::io;
use std::mem;

// TODO: indentation for wrapped lines
// TODO: counters for nested lists
// TODO: customizable list bullet
// TODO: omit bullet point for lists with checkboxes

pub(super) fn list(
    first_item_number: Option<u64>,
    events: &mut dyn Iterator<Item = Event<'_>>,
    state: &mut RenderState,
) -> io::Result<()> {
    state.write_block_start()?;

    let mut item_number = first_item_number;
    take! {
        for event in events; until Event::End(TagEnd::List(..)) => {
            if let Event::Start(Tag::Item) = event {
                item(item_number, events, state)?;
                item_number.as_mut().map(|c| *c += 1);
            } else {
                unreachable!();
            }
        }
    }

    Ok(())
}

fn item(
    number: Option<u64>,
    events: &mut dyn Iterator<Item = Event<'_>>,
    state: &mut RenderState,
) -> io::Result<()> {
    let mut fragments = Fragments::default();
    let scope = state.scope(state.style(), Some("   "));

    match number {
        Some(n) => fragments.push_text(&format!("{n}. ")),
        None => fragments.push_text("‒ "),
    }

    take! {
        for event in events; until Event::End(TagEnd::Item) => {
            if let Some(block_event) = fragments.try_push_event(event, state) {
                // If we get a block event, we only get block events from here on out.
                state.write_fragments(mem::take(&mut fragments), state.style())?;
                block(block_event, events, state)?;
            }
        }
    }

    state.write_fragments(fragments, state.style())?;
    state.end_scope(scope);

    Ok(())
}
