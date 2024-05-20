use pulldown_cmark::Parser;
use render::Options;
use std::fs;

mod counting;
mod fragment;
mod render;

// TODO: nonprintables
// TODO: trim trailing whitepace (ah I think that's why I had to add - 1 somehwere)
// TODO: hardbreaks

fn main() {
    let width = terminal_size::terminal_size()
        .map(|(width, _)| width.0)
        .unwrap_or(180);
    let input = fs::read_to_string("example.md").unwrap();
    let mut parser = Parser::new_ext(&input, parser_options());
    render::render(
        &mut parser,
        &mut std::io::stdout(),
        Options::plain_text(width),
    )
    .unwrap();
}

fn parser_options() -> pulldown_cmark::Options {
    use pulldown_cmark::Options;
    Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_STRIKETHROUGH
}

// TODO: Special rendering of list item after quote.
// TODO: `mdcat`-compatible CLI when run as `mdcat`.
// TODO: parindent.
