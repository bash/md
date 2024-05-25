use super::{Events, State};
use crate::fragment::Fragments;
use crate::prefix::Prefix;
use crate::render::fragment::FragmentsExt as _;
use anstyle::{AnsiColor, Style};
use pulldown_cmark::{Event, HeadingLevel, TagEnd};
use std::fmt::Write as _;
use std::io;

pub(super) fn heading(level: HeadingLevel, events: Events, state: &mut State) -> io::Result<()> {
    state.write_block_start()?;
    state.section_counter_mut().update(level);

    let prefix = Prefix::continued(numbering(state.section_counter().value()));

    state.block(
        |b| b.styled(|s| heading_style(s, level)).prefix(prefix),
        |state| {
            let mut fragments = Fragments::default();

            take! {
                for event in events; until Event::End(TagEnd::Heading(..)) => {
                    fragments.try_push_event(&event, state);
                }
            }

            state.write_fragments(fragments)
        },
    )
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

fn numbering<'b>(counters: &[usize]) -> String {
    let mut output = String::new();
    let counters = &counters[1..];

    // No numbering for sections with leading zeroes.
    if !counters.is_empty() && !counters.starts_with(&[0]) {
        for c in counters {
            write!(output, "{c}.").unwrap(); // TODO
        }
        write!(output, " ").unwrap(); // TODO
    }

    output
}
