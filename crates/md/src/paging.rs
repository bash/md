use std::env;
use std::io::{stdout, IsTerminal};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum PagingChoice {
    /// Enables paging only when the output is going to a terminal or TTY.
    Auto,
    /// Enables paging regardless of whether or not the output is going to a terminal/TTY.
    Always,
    /// Disables paging no matter if the output is going to a terminal/TTY, or not.
    Never,
}

impl PagingChoice {
    pub(crate) fn should_enable(&self) -> bool {
        match self {
            PagingChoice::Auto => stdout().is_terminal() && term_is_set_and_sensible(),
            PagingChoice::Always => true,
            PagingChoice::Never => false,
        }
    }
}

fn term_is_set_and_sensible() -> bool {
    env::var_os("TERM").is_some_and(|t| t != "dumb")
}
