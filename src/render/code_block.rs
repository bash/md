use super::{Events, State};
use crate::fmt_utils::Repeat;
use crate::syntax_highlighting::{highlight, Options};
use anstyle::{Reset, Style};
use pulldown_cmark::{CodeBlockKind, Event, TagEnd};
use std::io;
use unicode_width::UnicodeWidthStr;

pub(super) fn code_block(
    kind: CodeBlockKind<'_>,
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

    let language = match kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(language) => Some(language.into()),
    };

    let highlighted = highlight(
        &code,
        &Options {
            available_columns: state.available_columns(),
            language,
        },
    );

    // TODO: fix width calculation and re-enable
    // BoxWidget::write(&highlighted, state, Style::new().dimmed())

    for line in highlighted.lines() {
        state.write_prefix()?;
        writeln!(state.writer(), "{line}")?;
    }
    Ok(())
}

// TODO: pulldown-cmark's README breaks here, check why
struct BoxWidget;

impl BoxWidget {
    fn available_columns(available_columns: usize) -> usize {
        const BORDER_AND_PADDING_WIDTH: usize = 4;
        available_columns - BORDER_AND_PADDING_WIDTH
    }

    fn write(text: &str, state: &mut State, border_style: Style) -> io::Result<()> {
        let box_width = text.lines().map(UnicodeWidthStr::width).max().unwrap_or(0);
        state.write_prefix()?;
        writeln!(
            state.writer(),
            "{border_style}╭{}╮{Reset}",
            Repeat(box_width + 2, "─")
        )?;
        for line in text.lines() {
            let fill = box_width - line.width();
            state.write_prefix()?;
            writeln!(
                state.writer(),
                "{border_style}│{Reset} {line}{Reset}{f} {border_style}│{Reset}",
                f = Repeat(fill, " ")
            )?;
        }
        state.write_prefix()?;
        writeln!(
            state.writer(),
            "{border_style}╰{}╯{Reset}",
            Repeat(box_width + 2, "─")
        )?;
        Ok(())
    }
}
