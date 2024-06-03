use crate::context::Context;
use crate::inline::{InlineWriter, WritePrefixFn};
use crate::prefix::PrefixChain;
use std::io::{self, Write};

/// Wrapper around an [`Write`] with some convenience on top.
pub(crate) struct Writer<'w>(&'w mut dyn Write);

impl<'w> Writer<'w> {
    pub(crate) fn new(inner: &'w mut dyn Write) -> Self {
        Self(inner)
    }

    pub(crate) fn write_prefix(&mut self, ctx: &Context) -> io::Result<()> {
        write_prefix(ctx.prefix_chain(), self)
    }

    pub(super) fn write_blank_line(&mut self, ctx: &Context) -> io::Result<()> {
        self.write_prefix(ctx)?;
        writeln!(self)
    }

    /// Creates a temporary [`InlineWriter`] around this writer.
    /// The returned writer is used to write [`crate::inline::Inline`]s to the output.
    pub(crate) fn inline_writer<'a, 'p>(
        &mut self,
        ctx: &'p Context<'p, '_, '_>,
    ) -> InlineWriter<'a, '_, impl WritePrefixFn + 'p> {
        let prefix = ctx.prefix_chain();
        InlineWriter::new(ctx.style(), ctx.text_width(), self, move |w| {
            write_prefix(prefix, w)
        })
    }

    // TODO: Make this an actual margin control thing
    pub(crate) fn write_block_start(&mut self, b: &Context) -> io::Result<()> {
        if b.previous_block().is_some() {
            self.write_blank_line(b)?;
        }
        Ok(())
    }
}

impl Write for Writer<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        self.0.write_fmt(fmt)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }
}

fn write_prefix(prefix: &PrefixChain<'_>, w: &mut dyn Write) -> io::Result<()> {
    write!(w, "{}", prefix.display_next())
}
