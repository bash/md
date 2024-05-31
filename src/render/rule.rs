use super::context::BlockKind;
use super::{prelude::*, BlockRenderer};
use crate::fmt_utils::Repeat;

pub(super) struct Rule;

impl BlockRenderer for Rule {
    fn kind(&self) -> BlockKind {
        BlockKind::Rule
    }

    fn render(
        self,
        events: Events,
        state: &mut State,
        w: &mut Writer,
        b: super::context::BlockContext,
    ) -> io::Result<()> {
        // let decoration = "∗ ∗ ∗";
        // let padding_size = state
        //     .available_columns()
        //     .saturating_sub(display_width(decoration))
        //     / 2;

        // writeln!(
        //     state.writer(),
        //     "{pad}{decoration}{pad}",
        //     pad = Repeat(padding_size, " "),
        // )

        let columns = state.available_columns(&b).saturating_sub(2);
        w.write_prefix(&b)?;
        writeln!(w, "◈{}◈", Repeat(columns, "─"))
    }
}
