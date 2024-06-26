mod buffer;
mod display_width;
pub(crate) use display_width::DisplayWidth;
mod fragment;

#[cfg(test)]
mod tests;

use buffer::{BufferedChunk, ChunkBuffer};
use fragment::{Fragment, LinebreaksExt as _};
use pulldown_cmark::CowStr;
use trait_set::trait_set;
use unicode_linebreak_chunked::{BreakOpportunity, Linebreaks};
use unicode_width::UnicodeWidthStr as _;

/// A chunk of inline text that can be passed to the layouter.
#[derive(Debug, Clone)]
pub(crate) enum RawChunk<'a, P> {
    Text(CowStr<'a>),
    Passthrough(P),
}

impl<'a, P> RawChunk<'a, P> {
    pub(crate) fn soft_break() -> Self {
        RawChunk::Text(CowStr::from(" "))
    }

    pub(crate) fn hard_break() -> Self {
        RawChunk::Text(CowStr::from("\n"))
    }
}

/// A layouted chunk.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Chunk<'a, P> {
    /// Start of a line. Not emitted after the final newline.
    LineStart,
    /// Text to be written on a line.
    /// Note that a word can be split across multiple chunks
    /// with [`Chunk::Passthrough`]s in between.
    Text(DisplayWidth<CowStr<'a>>),
    /// Arbitrary data treated as zero-width by the layouter.
    /// Relative order among text and passthrough chunks is retained.
    Passthrough(P),
    /// End of a line.
    LineEnd,
}

impl<'a, P> Chunk<'a, P> {
    #[cfg(test)]
    pub(crate) fn text(s: impl Into<CowStr<'a>>) -> Self {
        Chunk::Text(DisplayWidth::from(s.into()))
    }

    #[cfg(test)]
    pub(crate) fn into_static(self) -> Chunk<'static, P> {
        match self {
            Chunk::LineStart => Chunk::LineStart,
            Chunk::Text(t) => Chunk::text(t.to_owned()), // This loses the cached display width, but that's ok for tests.
            Chunk::Passthrough(p) => Chunk::Passthrough(p),
            Chunk::LineEnd => Chunk::LineEnd,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ChunkLayouter<'a, P> {
    state: LineState,
    line_breaks: Linebreaks,
    buffer: ChunkBuffer<'a, P>,
}

impl<'a, P> ChunkLayouter<'a, P> {
    pub(crate) fn new(max_width: usize) -> Self {
        Self {
            state: LineState {
                max_width,
                used_width: 0,
            },
            line_breaks: Linebreaks::default(),
            buffer: ChunkBuffer::default(),
        }
    }
}

#[derive(Debug)]
struct LineState {
    max_width: usize,
    used_width: usize,
}

trait_set! {
    pub(crate) trait ChunkFn<'a, P, E> = FnMut(Chunk<'a, P>) -> Result<(), E>;
}

impl<'a, P> ChunkLayouter<'a, P> {
    pub(crate) fn chunk<E>(
        &mut self,
        chunk: RawChunk<'a, P>,
        f: impl for<'c> ChunkFn<'c, P, E>,
    ) -> Result<(), E> {
        match chunk {
            RawChunk::Text(s) => self.text(s, f),
            RawChunk::Passthrough(p) => self.passthrough(p, f),
        }
    }

    pub(crate) fn end<E>(&mut self, f: impl for<'c> ChunkFn<'c, P, E>) -> Result<(), E> {
        if let Some(opportunity) = self.line_breaks.eot() {
            yield_(
                CowStr::Borrowed(""),
                f,
                &mut self.state,
                &mut self.buffer,
                opportunity,
            )?;
        }
        Ok(())
    }

    fn text<E>(&mut self, s: CowStr<'a>, mut f: impl for<'c> ChunkFn<'c, P, E>) -> Result<(), E> {
        for fragment in self.line_breaks.fragments(s) {
            match fragment {
                Fragment::Complete(text, opportunity) => {
                    yield_(text, &mut f, &mut self.state, &mut self.buffer, opportunity)?;
                }
                Fragment::Partial(text) => {
                    self.buffer
                        .push(BufferedChunk::Text(DisplayWidth::from(text)));
                }
            }
        }
        Ok(())
    }

    fn passthrough<E>(&mut self, p: P, mut f: impl for<'c> ChunkFn<'c, P, E>) -> Result<(), E> {
        if self.buffer.is_empty() && self.state.used_width > 0 {
            f(Chunk::Passthrough(p))
        } else {
            self.buffer.push(BufferedChunk::Passthrough(p));
            Ok(())
        }
    }
}

// TODO: trim trailing whitespace of each line
fn yield_<'a, 's, P, E>(
    s: CowStr<'s>,
    mut f: impl for<'c> ChunkFn<'c, P, E>,
    state: &mut LineState,
    buffer: &mut ChunkBuffer<'a, P>,
    opportunity: BreakOpportunity,
) -> Result<(), E> {
    let s = DisplayWidth::from(s);
    let chunk_width = s.width();
    let total_width = buffer.display_width() + chunk_width;

    if state.used_width != 0 && state.used_width + total_width > state.max_width {
        f(Chunk::LineEnd)?;
        state.used_width = 0;
    }

    if state.used_width == 0 && total_width > 0 {
        f(Chunk::LineStart)?;
    }

    buffer.drain().try_for_each(|chunk| f(chunk.into()))?;

    // This is not strictly needed but simplifies our tests...
    if !s.is_empty() {
        f(Chunk::Text(s))?;
    }

    state.used_width += total_width;

    if opportunity == BreakOpportunity::Mandatory {
        f(Chunk::LineEnd)?;
        state.used_width = 0;
    }

    Ok(())
}
