use super::{Events, State};
use anstyle::{Reset, Style};
use pulldown_cmark::{CodeBlockKind, Event, TagEnd};
use std::io;

// TODO: syntax highlighting
pub(super) fn code_block(
    _kind: CodeBlockKind<'_>,
    events: Events,
    state: &mut State,
) -> io::Result<()> {
    state.write_block_start()?;

    // TODO: yes, yes we could use a buffer that only buffers until the next line...
    let mut code = String::new();
    take! {
        for event in events; until Event::End(TagEnd::CodeBlock) => {
            if let Event::Text(text) = event {
                code.push_str(&text);
            } else {
                // TODO: unreachable
                panic!("Unexpected event {:#?}", event)
            }
        }
    }

    for line in code.lines() {
        state.write_prefix()?;
        writeln!(state.writer(), "{}{line}{}", Style::new().italic(), Reset)?;
    }

    Ok(())
}
