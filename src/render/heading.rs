use super::RenderState;
use crate::fragment::{Fragment, Fragments};
use crate::render::fragment::FragmentsExt as _;
use anstyle::{AnsiColor, Style};
use pulldown_cmark::{Event, HeadingLevel, TagEnd};
use std::fmt::Write as _;
use std::io;

pub(super) fn heading(
    level: HeadingLevel,
    events: &mut dyn Iterator<Item = Event<'_>>,
    state: &mut RenderState,
) -> io::Result<()> {
    let mut fragments = Fragments::default();

    state.section_counter_mut().update(level);
    fragments.push(format_heading_counter(state.section_counter().value()));

    take! {
        for event in events; until Event::End(TagEnd::Heading(..)) => {
            fragments.try_push_event(event, state);
        }
    }

    state.write_block_start()?;
    state.write_fragments(fragments, heading_style(state.style(), level))
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

fn format_heading_counter<'a, 'b>(counters: &'a [usize]) -> Fragment<'b> {
    let mut output = String::new();

    if counters.len() >= 2 {
        for c in &counters[1..] {
            write!(output, "{c}.").unwrap(); // TODO
        }
        write!(output, " ").unwrap(); // TODO
    }

    Fragment::word(&output).into_owned()
}
