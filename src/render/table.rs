use super::context::{BlockContext, BlockKind};
use super::{prelude::*, BlockRenderer};
use anstyle::AnsiColor::Red;
use pulldown_cmark::Alignment;

pub(super) struct Table {
    pub(super) alignments: Vec<Alignment>,
}

impl BlockRenderer for Table {
    fn kind(&self) -> BlockKind {
        BlockKind::Table
    }

    fn render(
        self,
        events: Events,
        _state: &mut State,
        w: &mut Writer,
        b: &BlockContext,
    ) -> io::Result<()> {
        w.write_prefix(&b)?;
        writeln!(w, "{}[TODO: table]{}", Red.on_default().invert(), Reset)?;

        take! { for event in events; until Event::End(TagEnd::Table) => { } }

        Ok(())
    }
}
