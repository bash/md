use crate::hyperlink::{CloseHyperlink, Hyperlink};
use crate::style::{StyleExt as _, StyleStack};
use crate::textwrap::{Chunk, ChunkLayouter, RawChunk};
use anstyle::{Reset, Style};
use pulldown_cmark::CowStr;
use std::io;
use trait_set::trait_set;
use url::Url;

#[derive(Debug)]
pub(crate) enum Inline<'a> {
    Text(CowStr<'a>),
    SoftBreak,
    HardBreak,
    PushStyle(Style),
    PopStyle,
    SetLink(Url),
    UnsetLink,
}

impl<'a> From<Inline<'a>> for RawChunk<'a, Inline<'a>> {
    fn from(value: Inline<'a>) -> Self {
        match value {
            Inline::Text(text) => RawChunk::Text(text),
            Inline::SoftBreak => RawChunk::soft_break(),
            Inline::HardBreak => RawChunk::hard_break(),
            other => RawChunk::Passthrough(other),
        }
    }
}

impl<'a> From<CowStr<'a>> for Inline<'a> {
    fn from(value: CowStr<'a>) -> Self {
        Inline::Text(value)
    }
}

impl<'a> From<&'a str> for Inline<'a> {
    fn from(value: &'a str) -> Self {
        Inline::Text(CowStr::from(value))
    }
}

impl From<Style> for Inline<'_> {
    fn from(value: Style) -> Self {
        Inline::PushStyle(value)
    }
}

/// Writes an [`Inline`] and tracks the last used style.
///
/// This is useful when we want to render across multiple lines
/// when each line has a prefix with its own styling (e.g. blockquote, list).
pub(crate) struct InlineWriter<'a, 'w, F> {
    chunk_layouter: ChunkLayouter<'a, Inline<'a>>,
    state: WriterState<'w, F>,
}

trait_set! {
    pub(crate) trait WritePrefixFn = FnMut(&mut dyn io::Write) -> io::Result<()>;
}

struct WriterState<'w, F> {
    writer: &'w mut dyn io::Write,
    write_prefix: F,
    style_stack: StyleStack,
    link: Option<Url>,
    // The `id=` parameter helps the terminal connect links
    // that are broken up into multiple pieces (e.g. when we break the line)
    // https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
    link_id: usize,
}

impl<'a, 'w, F> InlineWriter<'a, 'w, F>
where
    F: WritePrefixFn,
{
    pub fn new(
        default_style: Style,
        max_width: usize,
        writer: &'w mut dyn io::Write,
        write_prefix: F,
    ) -> Self {
        Self {
            chunk_layouter: ChunkLayouter::new(max_width),
            state: WriterState {
                style_stack: StyleStack::new(default_style),
                link: None,
                link_id: 0,
                writer,
                write_prefix,
            },
        }
    }
}

impl<'a, 'w, F> InlineWriter<'a, 'w, F>
where
    F: WritePrefixFn,
{
    pub(crate) fn write(&mut self, inline: impl Into<Inline<'a>>) -> io::Result<()> {
        let raw_chunk = RawChunk::from(inline.into());
        self.chunk_layouter
            .chunk(raw_chunk, |chunk| write_chunk(chunk, &mut self.state))
    }

    pub(crate) fn end(mut self) -> io::Result<()> {
        self.chunk_layouter
            .end(|chunk| write_chunk(chunk, &mut self.state))
    }

    pub(crate) fn write_iter(
        &mut self,
        inlines: impl IntoIterator<Item = Inline<'a>>,
    ) -> io::Result<()> {
        inlines.into_iter().try_for_each(|i| self.write(i))
    }

    pub(crate) fn write_all(
        mut self,
        inlines: impl IntoIterator<Item = Inline<'a>>,
    ) -> io::Result<()> {
        self.write_iter(inlines)?;
        self.end()
    }
}

fn write_chunk<F>(chunk: Chunk<Inline<'_>>, ctx: &mut WriterState<'_, F>) -> io::Result<()>
where
    F: WritePrefixFn,
{
    match chunk {
        Chunk::LineStart => line_start(ctx),
        Chunk::Text(text) => write!(ctx.writer, "{text}"),
        Chunk::Passthrough(i) => inline(i, ctx),
        Chunk::LineEnd => line_end(ctx),
    }
}

fn line_start<F>(ctx: &mut WriterState<'_, F>) -> io::Result<()>
where
    F: WritePrefixFn,
{
    (ctx.write_prefix)(ctx.writer)?;
    write!(ctx.writer, "{}", ctx.style_stack.head())?;
    if let Some(url) = ctx.link.as_ref() {
        write!(ctx.writer, "{}", Hyperlink::new(url, ctx.link_id))?;
    }
    Ok(())
}

fn line_end<F>(ctx: &mut WriterState<'_, F>) -> io::Result<()> {
    write!(ctx.writer, "{}", Reset)?;
    if ctx.link.as_ref().is_some() {
        write!(ctx.writer, "{CloseHyperlink}")?;
    }
    writeln!(ctx.writer)
}

fn inline<F>(inline: Inline<'_>, ctx: &mut WriterState<'_, F>) -> io::Result<()> {
    let w = &mut *ctx.writer;
    let style_stack = &mut ctx.style_stack;
    match inline {
        Inline::Text(_) | Inline::SoftBreak | Inline::HardBreak => {
            unreachable!("these should not be passthrough")
        }
        Inline::PushStyle(s) => {
            style_stack.push(s.on_top_of(&style_stack.head()));
            write!(w, "{}", style_stack.head())
        }
        Inline::PopStyle => {
            style_stack.pop();
            write!(w, "{Reset}{}", style_stack.head())
        }
        Inline::SetLink(url) => {
            if ctx.link.is_some() {
                panic!("BUG: nested links"); // TODO: does pulldown-cmark support that?
            }
            ctx.link = Some(url.clone());
            ctx.link_id += 1;
            write!(w, "{}", Hyperlink::new(url, ctx.link_id))
        }
        Inline::UnsetLink => {
            // This must handle the case where the link was not pushed
            // but is popped as not all `Tag::Link`s result in links but all `TagEnd::Link`s do.
            // Sending a "link reset" when no link is open is perfectly fine.
            if ctx.link.take().is_some() {
                write!(w, "{CloseHyperlink}")?;
            }
            Ok(())
        }
    }
}
