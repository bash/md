use super::{Block, Events, State};
use crate::fragment::{Fragment, Fragments};
use crate::render::fragment::FragmentsExt as _;
use anstyle::{AnsiColor, Style};
use pulldown_cmark::{Event, HeadingLevel, TagEnd};
use std::fmt::Write as _;
use std::io;

pub(super) fn heading(level: HeadingLevel, events: Events, state: &mut State) -> io::Result<()> {
    state.write_block_start()?;
    state.section_counter_mut().update(level);

    let block = Block::default().with_style(heading_style(state.style(), level));
    state.block(block, |state| {
        let mut fragments = Fragments::default();

        fragments.push(format_heading_counter(state.section_counter().value()));

        take! {
            for event in events; until Event::End(TagEnd::Heading(..)) => {
                fragments.try_push_event(event, state);
            }
        }

        state.write_fragments(fragments)
    })
}

fn heading_style(style: Style, level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => style
            .fg_color(Some(AnsiColor::Green.into()))
            .bold()
            .underline(),
        HeadingLevel::H2 => style.fg_color(Some(AnsiColor::Green.into())).bold(),
        _ => style.fg_color(Some(AnsiColor::Green.into())),
    }
}

fn format_heading_counter<'b>(counters: &[usize]) -> Fragment<'b> {
    let mut output = String::new();

    if counters.len() >= 2 {
        for c in &counters[1..] {
            write!(output, "{c}.").unwrap(); // TODO
        }
        write!(output, " ").unwrap(); // TODO
    }

    Fragment::word(&output).into_owned()
}
