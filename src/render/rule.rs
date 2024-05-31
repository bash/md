use super::{prelude::*, BlockRenderer};
use crate::fmt_utils::Repeat;

pub(super) struct Rule;

impl BlockRenderer for Rule {
    fn looks_for_end_tag(&self) -> bool {
        false
    }

    fn kind(&self) -> BlockKind {
        BlockKind::Rule
    }

    fn render<'e>(
        self,
        _events: Events<'_, 'e, '_>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut Writer,
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

        let columns = ctx.available_width().saturating_sub(2);
        w.write_prefix(ctx)?;
        writeln!(w, "◈{}◈", Repeat(columns, "─"))
    }
}
