use super::prelude::*;
use crate::fmt_utils::Repeat;

pub(super) fn rule(state: &mut State, w: &mut Writer) -> io::Result<()> {
    w.write_block_start()?;

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

    let columns = state.available_columns(w).saturating_sub(2);
    w.write_prefix()?;
    writeln!(w, "◈{}◈", Repeat(columns, "─"))
}
