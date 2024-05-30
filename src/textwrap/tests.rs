use super::*;
use Passthrough::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Passthrough {
    A,
    B,
    C,
}

#[test]
fn emits_no_chunks_for_empty_raw_chunks() {
    assert!(layout(vec![]).is_empty());
}

#[test]
fn emits_no_chunks_for_one_or_more_empty_chunk() {
    for n in 0..10 {
        let raw_chunks = vec![RawChunk::Text("".into()); n];
        assert!(layout(raw_chunks).is_empty());
    }
}

#[test]
fn emits_no_chunks_if_only_passthrough() {
    let raw_chunks = vec![
        RawChunk::Passthrough(A),
        RawChunk::Passthrough(B),
        RawChunk::Passthrough(C),
    ];
    assert!(layout(raw_chunks).is_empty());
}

#[test]
fn emits_line_start_and_end() {
    let expected = vec![Chunk::LineStart, Chunk::text("foo"), Chunk::LineEnd];
    assert_eq!(expected, layout(vec![RawChunk::Text("foo".into())]));
}

#[test]
fn does_not_emit_line_start_for_passthrough_elements() {
    let raw_chunks = vec![
        RawChunk::Text("this_text_fills_up_the_line\n".into()),
        RawChunk::Passthrough(A),
    ];
    let expected = vec![
        Chunk::LineStart,
        Chunk::text("this_text_fills_up_the_line"),
        Chunk::Passthrough(A),
        Chunk::LineEnd,
    ];
    assert_eq!(expected, layout(raw_chunks));
}

#[test]
fn breaks_at_newline_characters() {
    let raw_chunks = vec![RawChunk::Text("foo\nbar\nbaz".into())];
    let expected = vec![
        Chunk::LineStart,
        Chunk::text("foo"),
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("bar"),
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("baz"),
        Chunk::LineEnd,
    ];
    assert_eq!(expected, layout(raw_chunks));
}

#[test]
fn passthrough_and_chunks_does_not_break_words() {
    let raw_chunks = vec![
        RawChunk::Passthrough(A),
        RawChunk::Text("this_is_a_".into()),
        RawChunk::Passthrough(B),
        RawChunk::Text("very_long_".into()),
        RawChunk::Passthrough(C),
        RawChunk::Text("text_that_is_definitely_too_long_for_this_line".into()),
    ];
    let expected = vec![
        Chunk::LineStart,
        Chunk::Passthrough(A),
        Chunk::text("this_is_a_"),
        Chunk::Passthrough(B),
        Chunk::text("very_long_"),
        Chunk::Passthrough(C),
        Chunk::text("text_that_is_definitely_too_long_for_this_line"),
        Chunk::LineEnd,
    ];
    assert_eq!(expected, layout(raw_chunks));
}

#[test]
fn breaks_long_lines_between_words() {
    let raw_chunks = vec![RawChunk::Text(
        "Lorem ipsum dolor sit amet, \
             consectetur adipiscing elit, sed do eiusmod tempor \
             incididunt ut labore et dolore magna aliqua."
            .into(),
    )];
    let expected = vec![
        Chunk::LineStart,
        Chunk::text("Lorem "),
        Chunk::text("ipsum "),
        Chunk::text("dolor "),
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("sit "),
        Chunk::text("amet, "),
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("consectetur "),
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("adipiscing "),
        Chunk::text("elit, "), // TODO: sed should fit here (it fits without the trailing whitespace)
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("sed "),
        Chunk::text("do "),
        Chunk::text("eiusmod "),
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("tempor "),
        Chunk::text("incididunt "),
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("ut "),
        Chunk::text("labore "),
        Chunk::text("et "),
        Chunk::text("dolore "),
        Chunk::LineEnd,
        Chunk::LineStart,
        Chunk::text("magna "),
        Chunk::text("aliqua."),
        Chunk::LineEnd,
    ];
    assert_eq!(expected, layout(raw_chunks));
}

fn layout(raw_chunks: Vec<RawChunk<'_, Passthrough>>) -> Vec<Chunk<'_, Passthrough>> {
    let mut chunks = Vec::new();
    let mut layouter = ChunkLayouter::new(20);
    for c in raw_chunks {
        _ = layouter.chunk::<()>(c, |c| {
            chunks.push(c.into_static());
            Ok(())
        });
    }
    _ = layouter.end::<()>(|c| {
        chunks.push(c.into_static());
        Ok(())
    });
    chunks
}
