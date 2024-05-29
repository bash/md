use super::display_width::DisplayWidth;
use super::Chunk;
use pulldown_cmark::CowStr;
use smallvec::SmallVec;

#[derive(Debug)]
pub(super) struct ChunkBuffer<'a, P> {
    buffer: SmallVec<[BufferedChunk<'a, P>; 8]>,
    display_width: usize,
}

impl<'a, P> Default for ChunkBuffer<'a, P> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            display_width: Default::default(),
        }
    }
}

impl<'a, P> ChunkBuffer<'a, P> {
    pub(super) fn push(&mut self, chunk: BufferedChunk<'a, P>) {
        if let BufferedChunk::Text(t) = &chunk {
            // TODO: deal with overflows?
            self.display_width += t.display_width();
        }
        self.buffer.push(chunk);
    }

    pub(super) fn drain<'s>(&'s mut self) -> impl Iterator<Item = BufferedChunk<'a, P>> + 's {
        self.buffer.drain(..)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub(super) fn display_width(&self) -> usize {
        self.display_width
    }
}

#[derive(Debug)]
pub(super) enum BufferedChunk<'a, P> {
    Text(DisplayWidth<CowStr<'a>>),
    Passthrough(P),
}

impl<'a, P> From<BufferedChunk<'a, P>> for Chunk<'a, P> {
    fn from(value: BufferedChunk<'a, P>) -> Self {
        match value {
            BufferedChunk::Text(t) => Chunk::Text(t),
            BufferedChunk::Passthrough(p) => Chunk::Passthrough(p),
        }
    }
}
