use super::{Events, State};
use anstyle::{Reset, Style};
use pulldown_cmark::{CodeBlockKind, Event, TagEnd};
use std::io::{self, Write as _};

// TODO: syntax highlighting
pub(super) fn code_block(
    _kind: CodeBlockKind<'_>,
    events: Events,
    state: &mut State,
) -> io::Result<()> {
    state.write_block_start()?;

    // TODO: allow writer to set/reset style
    let mut writer = state.writer();

    write!(writer.raw(), "{}", Style::new().italic())?;

    take! {
        for event in events; until Event::End(TagEnd::CodeBlock) => {
            if let Event::Text(text) = event {
                write!(writer, "{}", text)?;
            } else {
                // TODO: unreachable
                panic!("Unexpected event {:#?}", event)
            }
        }
    }

    write!(writer.raw(), "{}", Reset)?;

    Ok(())
}
