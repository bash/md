use matte::anstyle::{Reset, Style};
use matte::file_uri::{current_dir, file_in_current_dir};
use matte::pulldown_cmark::Parser;
use matte::url::Url;
use matte::{render, supported_parser_options, Options};
use output::Output;
use paging::PagingChoice;
use panic::setup_human_panic;
use std::io::{stdin, ErrorKind, IsTerminal, Read};
use std::path::Path;
use std::{env, fs};

mod file_detection;
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

    let width = terminal_size::terminal_size()
        .map(|(width, _)| width.0)
        .unwrap_or(180);
    let input = read_input();
    let mut parser = Parser::new_ext(&input.markdown, supported_parser_options());

    let mut output = Output::from_env(&input.file_name, PagingChoice::Auto).unwrap();
    let mut options = Options::plain_text(width);
    options.base_url = Some(input.base_url);
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

struct Input {
    markdown: String,
    file_name: String,
    base_url: Url,
}

// TODO: this needs to be moved and refactored
fn read_input() -> Input {
    match env::args_os().nth(1) {
        Some(path) => Input {
            markdown: fs::read_to_string(&path).unwrap(),
            file_name: Path::new(&path)
                .file_name()
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string(),
            base_url: file_in_current_dir(path).unwrap(),
        },
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
            Input {
                markdown: input,
                file_name: "STDIN".to_owned(),
                base_url: current_dir().unwrap(),
            }
        }
    }
}
