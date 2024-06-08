use super::prelude::*;
use crate::block::Block;
use anstyle::AnsiColor::Red;
use pulldown_cmark::Alignment;

pub(crate) struct Table {
    pub(crate) alignments: Vec<Alignment>,
}

impl Block for Table {
    fn kind(&self) -> BlockKind {
        BlockKind::Table
    }

    fn render<'e>(
        self,
        events: &mut impl Events<'e>,
        ctx: &Context<'_, 'e, '_>,
        mut w: &mut dyn Write,
    ) -> io::Result<()> {
        w.write_prefix(ctx)?;
        writeln!(w, "{}[TODO: table]{}", Red.on_default().invert(), Reset)?;

        terminated!(events, Event::End(TagEnd::Table)).for_each(|_event| {});

        Ok(())
    }
}
