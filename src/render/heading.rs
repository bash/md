use super::context::{BlockContext, BlockKind};
use super::{prelude::*, BlockRenderer};
use crate::prefix::Prefix;
use crate::render::inline::into_inlines;
use crate::style::StyledStr;
use anstyle::AnsiColor::Green;
use pulldown_cmark::HeadingLevel;
use std::fmt::Write as _;

pub(super) struct Heading {
    pub(super) level: HeadingLevel,
}

impl BlockRenderer for Heading {
    fn kind(&self) -> BlockKind {
        BlockKind::Heading(self.level)
    }

    fn render(
        self,
        events: Events,
        state: &mut State,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()> {
        state.section_counter_mut().update(self.level);

        let style = heading_style(self.level);
        let prefix = Prefix::continued(StyledStr::new(
            numbering(state.section_counter().value()),
            style,
        ));
        let b = b.child(prefix).styled(style);

        let mut writer = w.inline_writer(state, &b);

        take! {
            for event in events; until Event::End(TagEnd::Heading(..)) => {
                writer.write_iter(into_inlines(event, state))?;
            }
        }

        writer.end()
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => Green.on_default().bold().underline(),
        HeadingLevel::H2 => Green.on_default().bold(),
        _ => Green.on_default(),
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
