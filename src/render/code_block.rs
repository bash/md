use super::context::{BlockContext, BlockKind};
use super::{prelude::*, BlockRenderer};
use crate::syntax_highlighting::{highlight, Options};
use pulldown_cmark::CodeBlockKind;

pub(super) struct CodeBlock<'a> {
    pub(super) kind: CodeBlockKind<'a>,
}

impl BlockRenderer for CodeBlock<'_> {
    fn kind(&self) -> BlockKind {
        BlockKind::BlockQuote
    }

    fn render(
        self,
        events: Events,
        state: &mut State,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()> {
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

        let language = match self.kind {
            CodeBlockKind::Indented => None,
            CodeBlockKind::Fenced(language) => Some(language.into()),
        };

        let highlighted = highlight(
            &code,
            &Options {
                available_columns: state.available_columns(b),
                language,
            },
        );

        // TODO: fix width calculation and re-enable
        // BoxWidget::write(&highlighted, state, Style::new().dimmed())

        for line in highlighted.lines() {
            w.write_prefix(b)?;
            writeln!(w, "{line}")?;
        }
        Ok(())
    }
}

// TODO: pulldown-cmark's README breaks here, check why
// struct BoxWidget;

// impl BoxWidget {
//     fn available_columns(available_columns: usize) -> usize {
//         const BORDER_AND_PADDING_WIDTH: usize = 4;
//         available_columns - BORDER_AND_PADDING_WIDTH
//     }

//     fn write(text: &str, w: &mut Writer, border_style: Style) -> io::Result<()> {
//         let box_width = text.lines().map(UnicodeWidthStr::width).max().unwrap_or(0);
//         w.write_prefix()?;
//         writeln!(w, "{border_style}╭{}╮{Reset}", Repeat(box_width + 2, "─"))?;
//         for line in text.lines() {
//             let fill = box_width - line.width();
//             w.write_prefix()?;
//             writeln!(
//                 w,
//                 "{border_style}│{Reset} {line}{Reset}{f} {border_style}│{Reset}",
//                 f = Repeat(fill, " ")
//             )?;
//         }
//         w.write_prefix()?;
//         writeln!(w, "{border_style}╰{}╯{Reset}", Repeat(box_width + 2, "─"))?;
//         Ok(())
//     }
// }
