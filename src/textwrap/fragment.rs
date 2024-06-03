use pulldown_cmark::{CowStr, InlineStr};
use std::mem;
use unicode_linebreak_chunked::{BreakOpportunity, Linebreaks};

// A piece of text that doesn't contain
// any break opportunities.
#[derive(Debug)]
pub(super) enum Fragment<'a> {
    Complete(CowStr<'a>, BreakOpportunity),
    Partial(CowStr<'a>),
}

pub(super) trait LinebreaksExt {
    fn fragments<'a>(&mut self, s: CowStr<'a>) -> impl Iterator<Item = Fragment<'a>>;
}

impl LinebreaksExt for Linebreaks {
    fn fragments<'a>(&mut self, s: CowStr<'a>) -> impl Iterator<Item = Fragment<'a>> {
        FragmentsIter {
            input: s,
            linebreaks: self,
            start: 0,
            fragment_start: 0,
        }
    }
}

struct FragmentsIter<'l, 'a> {
    input: CowStr<'a>,
    start: usize,
    fragment_start: usize,
    linebreaks: &'l mut Linebreaks,
}

impl<'l, 'a> Iterator for FragmentsIter<'l, 'a> {
    type Item = Fragment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.linebreaks.chunk(&self.input, self.start) {
            Some((index, next_start, opportunity)) => {
                let fragment = fragment(&mut self.input, self.fragment_start, index);
                self.start = next_start;
                self.fragment_start = index;
                Some(Fragment::Complete(fragment, opportunity))
            }
            None if self.fragment_start < self.input.len() => {
                let end = self.input.len();
                let fragment = fragment(&mut self.input, self.fragment_start, end);
                self.start = self.input.len();
                self.fragment_start = self.input.len();
                Some(Fragment::Partial(fragment))
            }
            None => {
                self.start = self.input.len();
                self.fragment_start = self.input.len();
                None
            }
        }
    }
}

fn fragment<'a>(s: &mut CowStr<'a>, start: usize, end: usize) -> CowStr<'a> {
    let end = end - trailing_newline_len(&s[start..end]);
    slice(s, start, end)
}

fn slice<'a>(s: &mut CowStr<'a>, start: usize, end: usize) -> CowStr<'a> {
    match s {
        // If the range covers the entire input we know that we're not called again
        // so we take the entire CowStr...
        _ if start == 0 && end == s.len() => mem::replace(s, CowStr::Borrowed("")),
        CowStr::Boxed(b) => CowStr::from(b[start..end].to_owned()),
        CowStr::Borrowed(b) => CowStr::Borrowed(&b[start..end]),
        // This will always work because our new inlined string is always shorter or the same size
        CowStr::Inlined(i) => CowStr::Inlined(InlineStr::try_from(&i[start..end]).unwrap()),
    }
}

fn trailing_newline_len(s: &str) -> usize {
    if s.ends_with("\r\n") {
        2
    } else if s.ends_with('\n') {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Fragment::*;

    #[test]
    fn fragments_do_not_include_trailing_newlines() {
        let input = "foo\nbar\r\nbaz\n";
        let mut l = Linebreaks::default();

        assert_eq!(
            fragments_str(CowStr::Borrowed(input), &mut l).collect::<Vec<_>>(),
            &["foo", "bar", "baz"]
        );
    }

    fn fragments_str<'a>(
        s: CowStr<'a>,
        l: &'a mut Linebreaks,
    ) -> impl Iterator<Item = String> + 'a {
        l.fragments(s).map(|f| match f {
            Complete(s, _) => s.to_string(),
            Partial(s) => s.to_string(),
        })
    }
}
