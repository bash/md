use super::Context;
use crate::fmt_utils::Repeat;
use std::io::{self, Write as _};

pub(super) fn rule(ctx: &mut Context) -> io::Result<()> {
    ctx.write_block_start()?;

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

    let columns = ctx.available_columns().saturating_sub(2);
    writeln!(ctx.writer(), "◈{}◈", Repeat(columns, "─"))
}