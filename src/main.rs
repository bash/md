use anstyle::{Reset, Style};
use options::Options;
use output::Output;
use paging::PagingChoice;
use pulldown_cmark::Parser;
use render::default_parser_options;
use std::io::{stdin, ErrorKind, IsTerminal, Read};
use std::path::Path;
use std::{env, fs};

mod bullets;
mod counting;
mod display_width;
mod fmt_utils;
mod footnotes;
mod fragment;
mod hyperlink;
mod lookahead;
mod options;
mod output;
mod pager;
mod paging;
mod prefix;
mod render;
mod syntax_highlighting;

// TODO: nonprintables
// TODO: trim trailing whitespace (ah I think that's why I had to add - 1 somehwere)
// TODO: max text width
// TODO: `mdcat`-compatible CLI when run as `mdcat`.
// TODO: parindent.
fn main() {
    let width = terminal_size::terminal_size()
        .map(|(width, _)| width.0)
        .unwrap_or(180);
    let (input, input_name) = read_input();
    let mut parser = Parser::new_ext(&input, default_parser_options());

    let mut output = Output::from_env(&input_name, PagingChoice::Auto).unwrap();
    let mut options = Options::plain_text(width);
    options.hyperlinks = output.hyperlinks();
    options.columns = options
        .columns
        .saturating_sub(output.decoration_width() as u16); // TODO: integers

    match render::render(&mut parser, &mut output, options) {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
        Err(e) => panic!("{e:?}"),
    }
}

fn read_input() -> (String, String) {
    match env::args_os().nth(1) {
        Some(path) => (
            fs::read_to_string(&path).unwrap(),
            Path::new(&path)
                .file_name()
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string(),
        ),
        None => {
            let mut input = String::new();

            if stdin().is_terminal() {
                eprintln!(
                    "{}reading from standard input...{}",
                    Style::new().italic(),
                    Reset
                )
            }

            stdin().read_to_string(&mut input).unwrap();
            (input, "STDIN".to_owned())
        }
    }
}
