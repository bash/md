//! `>_` Matte—Your friendly neighbourhood **ma**rkdown **te**rminal renderer.

mod bullets;
mod chars;
mod counting;

mod fmt_utils;
mod footnotes;
mod hyperlink;
mod inline;
mod lookahead;
mod options;
mod prefix;
mod render;
mod style;
mod syntax_highlighting;
mod textwrap;

pub use options::*;
pub use render::*;
pub mod file_uri;

// Re-exports of crates that we use in our public API.
pub use pulldown_cmark;
pub use url;
