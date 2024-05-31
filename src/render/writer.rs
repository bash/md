use anstyle::Style;

use crate::context::Context;
use crate::fmt_utils::NoDebug;
use crate::inline::{InlineWriter, WritePrefixFn};
use crate::prefix::PrefixChain;
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

    pub(super) fn write_prefix(&mut self, ctx: &Context) -> io::Result<()> {
        write_prefix(ctx.prefix_chain(), ctx.style(), &mut *self.output)
    }

    pub(super) fn write_blank_line(&mut self, ctx: &Context) -> io::Result<()> {
        self.write_prefix(ctx)?;
        writeln!(self.output)
    }

    pub(super) fn inline_writer<'a, 'p>(
        &mut self,
        ctx: &'p Context<'p, '_, '_>,
    ) -> InlineWriter<'a, '_, impl WritePrefixFn + 'p> {
        let style = ctx.style();
        let prefix = ctx.prefix_chain();
        InlineWriter::new(
            ctx.style(),
            ctx.available_width(),
            &mut *self.output,
            move |w| write_prefix(prefix, style, w),
        )
    }

    pub(super) fn write_block_start(&mut self, b: &Context) -> io::Result<()> {
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

fn write_prefix(prefix: &PrefixChain<'_>, style: Style, w: &mut dyn io::Write) -> io::Result<()> {
    write!(w, "{}", prefix.display_next(style))
}
