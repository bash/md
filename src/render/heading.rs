use super::context::{BlockContext, BlockKind};
use super::{prelude::*, BlockRenderer};
use crate::counting::SectionCounter;
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
        state.counters().update_section(self.level);

        let style = heading_style(self.level);
        let prefix = Prefix::continued(numbering(state.counters().section()));
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
fn numbering(counters: SectionCounter) -> String {
    let mut output = String::new();
    let numbers = &counters.as_slice()[1..];

    // No numbering for sections with leading zeroes.
    if !numbers.is_empty() && !numbers.starts_with(&[0]) {
        for n in numbers {
            write!(output, "{n}.").unwrap(); // TODO
        }
        write!(output, " ").unwrap(); // TODO
    }

    output
}
