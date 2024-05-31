use super::hyperlink::{CloseHyperlink, Hyperlink};
use super::Inline;
use crate::style::{StyleExt as _, StyleStack};
use crate::textwrap::{Chunk, ChunkLayouter, RawChunk};
use anstyle::{Reset, Style};
use std::io;
use trait_set::trait_set;
use url::Url;

/// Writes an [`Inline`] and tracks the last used style.
///
/// This is useful when we want to render across multiple lines
/// when each line has a prefix with its own styling (e.g. blockquote, list).
pub(crate) struct InlineWriter<'a, 'w, F> {
    chunk_layouter: ChunkLayouter<'a, PassthroughInline>,
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

fn write_chunk<F>(chunk: Chunk<PassthroughInline>, ctx: &mut WriterState<'_, F>) -> io::Result<()>
where
    F: WritePrefixFn,
{
    match chunk {
        Chunk::LineStart => line_start(ctx),
        Chunk::Text(text) => write!(ctx.writer, "{text}"),
        Chunk::Passthrough(i) => passthrough_inline(i, ctx),
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

fn passthrough_inline<F>(
    inline: PassthroughInline,
    ctx: &mut WriterState<'_, F>,
) -> io::Result<()> {
    use PassthroughInline::*;

    let w = &mut *ctx.writer;
    let stack = &mut ctx.style_stack;
    match inline {
        PushStyle(s) => {
            stack.push(s.on_top_of(stack.head()));
            write!(w, "{}", stack.head())
        }
        PopStyle => {
            stack.pop();
            write!(w, "{Reset}{}", stack.head())
        }
        SetLink(url) => {
            if ctx.link.is_none() {
                ctx.link = Some(url.clone());
                ctx.link_id += 1;
                write!(w, "{}", Hyperlink::new(url, ctx.link_id))?;
            }
            Ok(())
        }
        UnsetLink => {
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

/// The subset of [`Inline`]s that get passed through
/// text wrapping unchanged.
#[derive(Debug)]
enum PassthroughInline {
    PushStyle(Style),
    PopStyle,
    SetLink(Url),
    UnsetLink,
}

impl<'a> From<Inline<'a>> for RawChunk<'a, PassthroughInline> {
    fn from(value: Inline<'a>) -> Self {
        match value {
            Inline::Text(text) => RawChunk::Text(text),
            Inline::SoftBreak => RawChunk::soft_break(),
            Inline::HardBreak => RawChunk::hard_break(),
            Inline::PushStyle(style) => RawChunk::Passthrough(PassthroughInline::PushStyle(style)),
            Inline::PopStyle => RawChunk::Passthrough(PassthroughInline::PopStyle),
            Inline::SetLink(url) => RawChunk::Passthrough(PassthroughInline::SetLink(url)),
            Inline::UnsetLink => RawChunk::Passthrough(PassthroughInline::UnsetLink),
        }
    }
}
