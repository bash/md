use super::Context;
use anstyle::{Reset, Style};
use pulldown_cmark::{CodeBlockKind, Event, TagEnd};
use std::io::{self, Write as _};

// TODO: syntax highlighting
pub(super) fn code_block(
    _kind: CodeBlockKind<'_>,
    events: &mut dyn Iterator<Item = Event<'_>>,
    ctx: &mut Context,
) -> io::Result<()> {
    ctx.write_block_start()?;

    // TODO: allow writer to set/reset style
    let mut writer = ctx.writer();

    write!(writer.raw(), "{}", Style::new().italic())?;

    take! {
        for event in events; until Event::End(TagEnd::CodeBlock) => {
            if let Event::Text(text) = event {
                write!(writer, "{}", text)?;
            } else {
                panic!("Unexpected event {:#?}", event)
            }
        }
    }

    write!(writer.raw(), "{}", Reset)?;

    Ok(())
}
