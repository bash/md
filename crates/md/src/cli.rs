use clap::Parser;

use crate::input::InputArg;

#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Args {
    pub(crate) input: InputArg,
}
