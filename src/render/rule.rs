use super::RenderState;
use crate::fmt_utils::Repeat;
use std::io::{self, Write as _};
use textwrap::core::display_width;

pub(super) fn rule(state: &mut RenderState) -> io::Result<()> {
    state.write_block_start()?;

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

    let columns = state.available_columns().saturating_sub(2);
    writeln!(state.writer(), "◈{}◈", Repeat(columns, "─"))
}
