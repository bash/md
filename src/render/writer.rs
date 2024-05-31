use super::context::BlockContext;
use super::state::State;
use crate::fmt_utils::NoDebug;
use crate::inline::{InlineWriter, WritePrefixFn};
use std::io;

#[derive(Debug)]
pub(super) struct Writer<'w> {
    output: NoDebug<&'w mut dyn io::Write>,
}

impl<'w> Writer<'w> {
    pub(super) fn new(output: &'w mut dyn io::Write) -> Self {
        Self {
            output: NoDebug(output),
        }
    }

    pub(super) fn write_prefix(&mut self, b: &BlockContext) -> io::Result<()> {
        write_prefix(b, &mut *self.output)
    }

    pub(super) fn write_blank_line(&mut self, b: &BlockContext) -> io::Result<()> {
        write_prefix(b, &mut *self.output)?;
        writeln!(self.output)
    }

    pub(super) fn inline_writer<'i, 's>(
        &'s mut self,
        state: &State,
        block: &'s BlockContext,
    ) -> InlineWriter<'i, 's, impl WritePrefixFn + 's> {
        InlineWriter::new(
            block.style(),
            state.text_columns(block),
            &mut *self.output,
            |w| write_prefix(block, w),
        )
    }

    pub(super) fn write_block_start(&mut self, b: &BlockContext) -> io::Result<()> {
        if b.previous_block().is_some() {
            self.write_blank_line(b)?;
        }
        Ok(())
    }
}

impl io::Write for Writer<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.output.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        self.output.write_fmt(fmt)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.output.write_vectored(bufs)
    }
}

fn write_prefix(block: &BlockContext, w: &mut dyn io::Write) -> io::Result<()> {
    if let Some(parent) = block.parent() {
        write_prefix(parent, w)?;
    }

    write!(w, "{}", block.take_prefix())
}
