use crate::input::InputArg;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Args {
    pub(crate) input: InputArg,
}
