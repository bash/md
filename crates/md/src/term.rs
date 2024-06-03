use std::env;

pub(crate) fn term_is_set_and_sensible() -> bool {
    env::var_os("TERM").is_some_and(|t| t != "dumb")
}
