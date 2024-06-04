use clap::Parser as _;
use cli::Args;
use matte::pulldown_cmark::Parser;
use matte::{render, supported_parser_options, Options};
use output::Output;
use pager::Pager;
use paging::PagingChoice;
use panic::setup_human_panic;
use std::io::ErrorKind;

mod cli;
mod file_detection;
mod input;
mod output;
mod pager;
mod paging;
mod panic;
mod term;

// TODO: nonprintables
// TODO: trim trailing whitespace (ah I think that's why I had to add - 1 somehwere)
// TODO: max text width
// TODO: `mdcat`-compatible CLI when run as `mdcat`.
// TODO: parindent.
fn main() {
    setup_human_panic();

    let args = Args::parse();
    let mut markdown = String::new();
    let mut input = args.input.open().unwrap();
    input.read_to_string(&mut markdown).unwrap();

    let width = terminal_size::terminal_size()
        .map(|(width, _)| width.0)
        .unwrap_or(180);
    let mut parser = Parser::new_ext(&markdown, supported_parser_options());

    let mut output = Output::from_env(&input.name().to_string_lossy(), PagingChoice::Auto).unwrap();
    let mut options = Options::plain_text(width);
    options.base_url = Some(input.base_url().unwrap());
    options.hyperlinks = output.hyperlinks();
    options.columns = options
        .columns
        .saturating_sub(output.decoration_width() as u16); // TODO: integers

    match render(&mut parser, &mut output, options) {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
        Err(e) => panic!("{e:?}"),
    }
}
