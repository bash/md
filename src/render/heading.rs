use super::prelude::*;
use crate::prefix::Prefix;
use crate::render::inline::into_inlines;
use anstyle::AnsiColor::Green;
use pulldown_cmark::HeadingLevel;
use std::fmt::Write as _;

pub(super) fn heading(
    level: HeadingLevel,
    events: Events,
    state: &mut State,
    w: &mut Writer,
) -> io::Result<()> {
    state.section_counter_mut().update(level);
    w.write_block_start()?;

    let prefix = Prefix::continued(numbering(state.section_counter().value()));

    w.block(
        |b| b.styled(|s| heading_style(s, level)).prefix(prefix),
        |w| {
            let mut writer = w.inline_writer(state);

            take! {
                for event in events; until Event::End(TagEnd::Heading(..)) => {
                    writer.write_iter(into_inlines(event, state))?;
                }
            }

            writer.end()
        },
    )
}

fn heading_style(style: Style, level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => style.fg_color(Some(Green.into())).bold().underline(),
        HeadingLevel::H2 => style.fg_color(Some(Green.into())).bold(),
        _ => style.fg_color(Some(Green.into())),
    }
}

// TODO: having numbering for changelog files is really not nice
// Since this needs to be configurable anyways maybe we can have a heuristic
// that detects changelog files by name (any or no extension):
// * changelog, CHANGELOG, RELEASE_NOTES, releasenotes, RELEASENOTES
// others?
fn numbering(counters: &[usize]) -> String {
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
