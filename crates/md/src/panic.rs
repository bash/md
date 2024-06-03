use crate::term::term_is_set_and_sensible;
use human_panic::{metadata, setup_panic};
use std::env;

pub(crate) fn setup_human_panic() {
    setup_panic!(metadata!().support(support_text()));
}

fn support_text() -> &'static str {
    if term_is_set_and_sensible() {
        "- Open an issue on GitHub: \x1b]8;;https://github.com/bash/md/issues/new\x1b\\https://github.com/bash/md/issues/new\x1b]8;;\x1b\\"
    } else {
        "- Open an issue on GitHub: https://github.com/bash/md/issues/new"
    }
}
