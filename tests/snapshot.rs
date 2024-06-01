use insta::{assert_snapshot, glob};
use matte::{render, supported_parser_options, Options};
use pulldown_cmark::{Event, MetadataBlockKind, Options as ParserOptions, Parser, Tag};
use serde::Deserialize;
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
    let snippet_options = read_snippet_options(input);
    let options = Options::plain_text(snippet_options.width.unwrap_or(120));
    render(parser, &mut buffer, options).unwrap();
    String::from_utf8(buffer).unwrap()
}

fn read_snippet_options(input: &str) -> SnippetOptions {
    toml::de::from_str(&extract_frontmatter(input)).unwrap()
}

fn extract_frontmatter(input: &str) -> String {
    let parser = Parser::new_ext(
        input,
        ParserOptions::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS,
    );
    let mut frontmatter = String::new();
    let mut inside_frontmatter = false;
    for event in parser {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::PlusesStyle)) => {
                inside_frontmatter = true
            }
            Event::Text(text) if inside_frontmatter => frontmatter.push_str(&text),
            _ => break,
        }
    }
    frontmatter
}

#[derive(Deserialize)]
struct SnippetOptions {
    width: Option<u16>,
}
