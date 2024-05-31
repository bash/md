use super::context::{BlockContext, BlockKind};
use super::{prelude::*, BlockRenderer};
use crate::inline::into_inlines;
use crate::prefix::Prefix;
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

    fn render<'e>(
        self,
        events: Events<'_, 'e, '_>,
        state: &mut State<'e>,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()> {
        state.section_counter_mut().update(self.level);

        let style = heading_style(self.level);
        let prefix = Prefix::continued(numbering(state.section_counter().value()));
        let b = b.child(prefix).styled(style);

        let writer = w.inline_writer(state, &b);
        writer.write_all(
            terminated!(events, Event::End(TagEnd::Heading(..)))
                .flat_map(|event| into_inlines(event, state)),
        )
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
