use crate::hyperlink::{CloseHyperlink, Hyperlink};
use crate::style::{StyleExt as _, StyleStack};
use crate::textwrap::{Chunk, ChunkLayouter, RawChunk};
use anstyle::{Reset, Style};
use pulldown_cmark::CowStr;
use std::io;
use trait_set::trait_set;
use url::Url;

/// TODO: rename to `Inline`.
#[derive(Debug)]
pub(crate) enum Fragment<'a> {
    Text(CowStr<'a>),
    SoftBreak,
    HardBreak,
    PushStyle(Style),
    PopStyle,
    SetLink(Url),
    UnsetLink,
}

impl<'a> From<Fragment<'a>> for RawChunk<'a, Fragment<'a>> {
    fn from(value: Fragment<'a>) -> Self {
        match value {
            Fragment::Text(text) => RawChunk::Text(text),
            Fragment::SoftBreak => RawChunk::soft_break(),
            Fragment::HardBreak => RawChunk::hard_break(),
            other => RawChunk::Passthrough(other),
        }
    }
}

impl<'a> From<CowStr<'a>> for Fragment<'a> {
    fn from(value: CowStr<'a>) -> Self {
        Fragment::Text(value)
    }
}

impl<'a> From<&'a str> for Fragment<'a> {
    fn from(value: &'a str) -> Self {
        Fragment::Text(CowStr::from(value))
    }
}

impl From<Style> for Fragment<'_> {
    fn from(value: Style) -> Self {
        Fragment::PushStyle(value)
    }
}

/// Writes a fragment and tracks the last used style.
///
/// This is useful when we want to render across multiple lines
/// when each line has a prefix with its own styling (e.g. blockquote, list).
pub(crate) struct FragmentWriter<'a, 'w, F> {
    chunk_layouter: ChunkLayouter<'a, Fragment<'a>>,
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

impl<'a, 'w, F> FragmentWriter<'a, 'w, F>
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

impl<'a, 'w, F> FragmentWriter<'a, 'w, F>
where
    F: WritePrefixFn,
{
    pub(crate) fn write(&mut self, fragment: impl Into<Fragment<'a>>) -> io::Result<()> {
        let raw_chunk = RawChunk::from(fragment.into());
        self.chunk_layouter
            .chunk(raw_chunk, |chunk| write_chunk(chunk, &mut self.state))
    }

    pub(crate) fn end(&mut self) -> io::Result<()> {
        self.chunk_layouter
            .end(|chunk| write_chunk(chunk, &mut self.state))
    }

    pub(crate) fn write_iter(
        &mut self,
        fragments: impl IntoIterator<Item = Fragment<'a>>,
    ) -> io::Result<()> {
        fragments.into_iter().try_for_each(|f| self.write(f))
    }
}

fn write_chunk<F>(chunk: Chunk<Fragment<'_>>, ctx: &mut WriterState<'_, F>) -> io::Result<()>
where
    F: WritePrefixFn,
{
    match chunk {
        Chunk::LineStart => line_start(ctx),
        Chunk::Text(text) => write!(ctx.writer, "{text}"),
        Chunk::Passthrough(f) => fragment(f, ctx),
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
    if let Some(_) = ctx.link.as_ref() {
        write!(ctx.writer, "{CloseHyperlink}")?;
    }
    writeln!(ctx.writer)
}

fn fragment<F>(fragment: Fragment<'_>, ctx: &mut WriterState<'_, F>) -> io::Result<()> {
    let w = &mut *ctx.writer;
    let style_stack = &mut ctx.style_stack;
    match fragment {
        Fragment::Text(_) | Fragment::SoftBreak | Fragment::HardBreak => {
            unreachable!("these should not be passthrough")
        }
        Fragment::PushStyle(s) => {
            style_stack.push(s.on_top_of(&style_stack.head()));
            write!(w, "{}", style_stack.head())
        }
        Fragment::PopStyle => {
            style_stack.pop();
            write!(w, "{Reset}{}", style_stack.head())
        }
        Fragment::SetLink(url) => {
            if ctx.link.is_some() {
                panic!("BUG: nested links"); // TODO: does pulldown-cmark support that?
            }
            ctx.link = Some(url.clone());
            ctx.link_id += 1;
            write!(w, "{}", Hyperlink::new(url, ctx.link_id))
        }
        Fragment::UnsetLink => {
            // This must handle the case where the link was not pushed
            // but is popped as not all `Tag::Link`s result in links but all `TagEnd::Link`s do.
            // Sending a "link reset" when no link is open is perfectly fine.
            ctx.link = None;
            write!(w, "{CloseHyperlink}")
        }
    }
}
