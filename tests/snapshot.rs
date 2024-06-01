use insta::{assert_snapshot, glob};
use matte::{render, supported_parser_options, Options};
use pulldown_cmark::Parser;
use std::fs::read_to_string;

#[test]
fn test_snippets() {
    glob!("snippets/*.md", |path| {
        let markdown = read_to_string(path).unwrap();
        let rendered = render_to_string(&markdown);
        assert_snapshot!(rendered);
    })
}

fn render_to_string(input: &str) -> String {
    let parser = Parser::new_ext(input, supported_parser_options());
    let mut buffer = Vec::new();
    let options = Options::plain_text(120);
    render(parser, &mut buffer, options).unwrap();
    String::from_utf8(buffer).unwrap()
}
