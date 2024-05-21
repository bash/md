use crate::counting::SectionCounter;
use crate::fmt_utils::Repeat;
use crate::fragment::{Fragment, FragmentWriter, Fragments, Word};
use crate::options::Options;
use anstyle::{AnsiColor, Reset, Style};
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd, TextMergeStream};
use std::io::{self, Write};
use std::{iter, mem};
use textwrap::core::display_width;

pub fn render<'a, I, W>(input: &mut I, output: &mut W, options: Options) -> io::Result<()>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    let mut events = TextMergeStream::new(input);
    let mut state = RenderState::default();

    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::Paragraph) => on_block_start(&mut state, output)?,
            Event::End(TagEnd::Paragraph) => {
                flush_fragments(&mut state, output, &options)?;
                on_block_end(&mut state)?;
            }

            Event::Start(Tag::Heading { level, .. }) => {
                on_block_start(&mut state, output)?;
                state.section_counter.update(level);
                state.stack.update_style(|s| heading_style(s, level));
            }
            Event::End(TagEnd::Heading(_)) => {
                write!(output, "{}", state.stack.style())?;
                write_heading_counter(&mut state, output)?;
                flush_fragments(&mut state, output, &options)?;
                write!(output, "{}", Reset)?;
                on_block_end(&mut state)?;
            }

            Event::Start(Tag::MetadataBlock(_)) => {
                state.text_mode = TextMode::Discard;
            }

            Event::End(TagEnd::MetadataBlock(_)) => {
                state.text_mode = TextMode::default();
            }

            Event::Start(Tag::CodeBlock(_)) => {
                on_block_start(&mut state, output)?;
                state.text_mode = TextMode::Code;
            }

            Event::End(TagEnd::CodeBlock) => {
                let code = mem::take(&mut state.code);
                state.text_mode = TextMode::default();
                write!(output, "{}", AnsiColor::Yellow.on_default())?;
                for line in code.lines() {
                    write_line_prefix(&mut state, output)?;
                    write!(output, "{}", line)?;
                    writeln!(output)?;
                }
                write!(output, "{}", Reset)?;
                on_block_end(&mut state)?;
            }

            Event::Start(Tag::List(..)) => {
                on_block_start(&mut state, output)?;
            }
            Event::End(TagEnd::List(..)) => {
                on_block_end(&mut state)?;
            }
            Event::Start(Tag::Item) => {}
            Event::End(TagEnd::Item) => flush_fragments(&mut state, output, &options)?,
            Event::Start(Tag::Table(..)) => {
                on_block_start(&mut state, output)?;
                state.text_mode = TextMode::Discard;
            }
            Event::End(TagEnd::Table) => {
                state.text_mode = TextMode::default();
                writeln!(
                    output,
                    "{red}[here be {strike}dragons{reset}{red} tables]{reset}",
                    red = AnsiColor::Red.on_default(),
                    strike = Style::new().strikethrough(),
                    reset = Reset
                )?;
                on_block_end(&mut state)?;
            }
            Event::Start(Tag::TableHead | Tag::TableRow | Tag::TableCell) => {}
            Event::End(TagEnd::TableHead | TagEnd::TableRow | TagEnd::TableCell) => {}

            Event::HardBreak => flush_fragments(&mut state, output, &options)?,

            Event::Start(Tag::FootnoteDefinition(name)) => {
                // TODO
                state
                    .fragments
                    .push(Word::new(&format!("[^{name}]: ")).into_owned());
            }
            Event::End(TagEnd::FootnoteDefinition) => {}

            Event::Start(Tag::Link { .. }) => {}
            Event::End(TagEnd::Link) => {}
            Event::FootnoteReference(_) => {}

            Event::Start(Tag::HtmlBlock) => {}
            Event::End(TagEnd::HtmlBlock) => {}
            Event::Html(_) => {}
            Event::Start(Tag::BlockQuote) => {
                on_block_start(&mut state, output)?;
                state.stack.push();
                state.stack.set_line_prefix("┃ ");
            }
            Event::End(TagEnd::BlockQuote) => {
                state.stack.pop();
                state.stack.on_block_encountered();
            }
            Event::Start(Tag::Image { .. }) => {
                state.fragments.push(Style::new().invert());
                state.fragments.push(Word::new("🖼 "));
            }
            Event::End(TagEnd::Image) => {
                state.fragments.push(Fragment::PopStyle);
            }
            Event::SoftBreak => {
                state.fragments.push(Fragment::SoftBreak);
            }
            Event::Text(t) => match state.text_mode {
                TextMode::Fragment => state
                    .fragments
                    .extend(Fragment::from_str(&t).map(Fragment::into_owned)),
                TextMode::Code => state.code.push_str(&t),
                TextMode::Discard => {}
            },
            Event::Code(code) => {
                state
                    .fragments
                    .push(AnsiColor::Yellow.on_default().italic());
                state
                    .fragments
                    .extend(Fragment::from_str(&code).map(Fragment::into_owned));
                state.fragments.push(Fragment::PopStyle);
            }
            Event::Start(Tag::Strong) => state.fragments.push(Style::new().bold()),
            Event::End(TagEnd::Strong) => state.fragments.push(Fragment::PopStyle),
            Event::Start(Tag::Emphasis) => state.fragments.push(Style::new().italic()),
            Event::End(TagEnd::Emphasis) => state.fragments.push(Fragment::PopStyle),
            Event::Start(Tag::Strikethrough) => state.fragments.push(Style::new().strikethrough()),
            Event::End(TagEnd::Strikethrough) => state.fragments.push(Fragment::PopStyle),
            Event::Rule => {
                on_block_start(&mut state, output)?;
                let decoration = "∗ ∗ ∗";
                let padding_size = available_columns(&mut state, &options)
                    .saturating_sub(display_width(decoration))
                    / 2;
                write!(output, "{0}{1}{0}", Repeat(padding_size, " "), decoration)?;
                writeln!(output)?;
                on_block_end(&mut state)?;
            }
            _ => todo!("{:?}", event),
        };
    }
    Ok(())
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

fn on_block_start<W: Write>(state: &mut RenderState<'_>, output: &mut W) -> io::Result<()> {
    if !state.stack.is_first_block() {
        write_line_prefix(state, output)?;
        writeln!(output)?;
    }
    state.stack.push();
    Ok(())
}

fn on_block_end(state: &mut RenderState<'_>) -> io::Result<()> {
    state.stack.pop();
    state.stack.on_block_encountered();
    Ok(())
}

fn write_line_prefix(state: &mut RenderState<'_>, output: &mut dyn io::Write) -> io::Result<()> {
    for prefix in state.stack.line_prefixes() {
        write!(output, "{}", prefix)?;
    }
    Ok(())
}

fn write_heading_counter<W: Write>(state: &mut RenderState<'_>, output: &mut W) -> io::Result<()> {
    let counters = state.section_counter.value();
    if counters.len() >= 2 {
        for c in &counters[1..] {
            write!(output, "{c}.")?;
        }
        write!(output, " ")?;
    }
    Ok(())
}

fn available_columns(state: &mut RenderState<'_>, options: &Options) -> usize {
    let prefix_len = state
        .stack
        .line_prefixes()
        .map(display_width)
        .sum::<usize>();
    (options.columns as usize).saturating_sub(prefix_len)
}

fn flush_fragments(
    state: &mut RenderState<'_>,
    output: &mut impl io::Write,
    options: &Options,
) -> io::Result<()> {
    let fragments = state.fragments.take();
    let mut writer = FragmentWriter::new(state.stack.style());
    writer.write_block(
        fragments,
        available_columns(state, options).saturating_sub(1),
        output,
        |w| write_line_prefix(state, w),
    )?;

    Ok(())
}

#[derive(Debug, Default)]
struct RenderState<'a> {
    text_mode: TextMode,
    fragments: Fragments<'a>,
    code: String,
    stack: BlockStack,
    section_counter: SectionCounter,
}

#[derive(Debug, Default)]
enum TextMode {
    #[default]
    Fragment,
    Code,
    Discard,
}

#[derive(Debug, Default)]
struct BlockStack {
    head: Block,
    tail: Vec<Block>,
}

impl BlockStack {
    fn line_prefixes(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.tail
            .iter()
            .rev()
            .map(|b| b.line_prefix)
            .chain(iter::once(self.head.line_prefix))
    }

    fn style(&self) -> Style {
        self.head.style
    }

    fn update_style(&mut self, f: impl FnOnce(Style) -> Style) {
        let base_style = self.tail.last().map(|b| b.style).unwrap_or_default();
        self.head.style = f(base_style);
    }

    fn is_first_block(&self) -> bool {
        self.head.is_first_block
    }

    fn set_line_prefix(&mut self, prefix: &'static str) {
        self.head.line_prefix = prefix;
    }

    fn on_block_encountered(&mut self) {
        self.head.is_first_block = false;
    }

    fn push(&mut self) {
        let old_head = mem::replace(&mut self.head, Block::default());
        self.head.style = old_head.style;
        self.tail.push(old_head);
    }

    fn pop(&mut self) {
        self.head = self.tail.pop().expect("failed to pop: stack is empty");
    }
}

#[derive(Debug)]
struct Block {
    line_prefix: &'static str,
    is_first_block: bool,
    style: Style,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            line_prefix: "",
            is_first_block: true,
            style: Style::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn renders_single_paragraph() {
        assert_eq!(
            "hello world",
            render_to_string("hello world", Options::plain_text(180))
        );
    }

    #[test]
    fn renders_single_paragraph_with_soft_break() {
        assert_eq!(
            "hello world",
            render_to_string("hello\nworld", Options::plain_text(180))
        );
    }

    fn render_to_string(input: &str, options: Options) -> String {
        let mut parser = Parser::new_ext(&input, parser_options());
        let mut output = Vec::new();
        render(&mut parser, &mut output, options).unwrap();
        return String::from_utf8(output).unwrap();
    }

    fn parser_options() -> pulldown_cmark::Options {
        use pulldown_cmark::Options;
        Options::ENABLE_FOOTNOTES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_TABLES
            | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
    }
}
