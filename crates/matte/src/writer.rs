use crate::context::Context;
use crate::inline::{InlineWriter, WritePrefixFn};
use crate::prefix::PrefixChain;
use std::io::{self, Write};

pub(crate) trait WriteExt {
    fn write_prefix(&mut self, ctx: &Context) -> io::Result<()>;

    fn write_blank_line(&mut self, ctx: &Context) -> io::Result<()>;

    fn inline_writer<'a, 'p>(
        &mut self,
        ctx: &'p Context<'p, '_, '_>,
    ) -> InlineWriter<'a, '_, impl WritePrefixFn + 'p>;

    fn write_block_start(&mut self, b: &Context) -> io::Result<()>;
}

impl<W: io::Write> WriteExt for W {
    fn write_prefix(&mut self, ctx: &Context) -> io::Result<()> {
        write_prefix(ctx.prefix_chain(), self)
    }

    fn write_blank_line(&mut self, ctx: &Context) -> io::Result<()> {
        self.write_prefix(ctx)?;
        writeln!(self)
    }

    /// Creates a temporary [`InlineWriter`] around this writer.
    /// The returned writer is used to write [`crate::inline::Inline`]s to the output.
    fn inline_writer<'a, 'p>(
        &mut self,
        ctx: &'p Context<'p, '_, '_>,
    ) -> InlineWriter<'a, '_, impl WritePrefixFn + 'p> {
        let prefix = ctx.prefix_chain();
        InlineWriter::new(ctx.style(), ctx.text_width(), self, move |w| {
            write_prefix(prefix, w)
        })
    }

    // TODO: Make this an actual margin control thing
    fn write_block_start(&mut self, b: &Context) -> io::Result<()> {
        if b.previous_block().is_some() {
            self.write_blank_line(b)?;
        }
        Ok(())
    }
}

fn write_prefix(prefix: &PrefixChain<'_>, w: &mut dyn Write) -> io::Result<()> {
    write!(w, "{}", prefix.display_next())
}
