use options::Options;
use pulldown_cmark::Parser;
use std::fs;
use std::io::ErrorKind;

mod bullets;
mod counting;
mod display_width;
mod fmt_utils;
mod footnotes;
mod fragment;
mod lookahead;
mod options;
mod prefix;
mod render;
mod syntax_highlighting;

// TODO: nonprintables
// TODO: trim trailing whitepace (ah I think that's why I had to add - 1 somehwere)
// TODO: max text width

fn main() {
    let arg = std::env::args_os().nth(1).expect("argument");
    let width = terminal_size::terminal_size()
        .map(|(width, _)| width.0)
        .unwrap_or(180);
    let input = fs::read_to_string(arg).unwrap();
    let mut parser = Parser::new_ext(&input, parser_options());

    match render::render(
        &mut parser,
        &mut std::io::stdout(),
        Options::plain_text(width),
    ) {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
        Err(e) => panic!("{e:?}"),
    }
}

fn parser_options() -> pulldown_cmark::Options {
    use pulldown_cmark::Options;
    Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_MATH
        | Options::ENABLE_GFM // Enables admonitions i.e. [!NOTE], ...
}

// TODO: Special rendering of list item after quote.
// TODO: `mdcat`-compatible CLI when run as `mdcat`.
// TODO: parindent.
