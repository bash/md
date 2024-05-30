use std::iter;
use std::ops::Range;
use unicode_linebreak_chunked::{BreakOpportunity, Linebreaks};

// A piece of text that doesn't contain
// any break opportunities.
#[derive(Debug)]
pub(super) enum Fragment {
    Complete(Range<usize>, BreakOpportunity),
    Partial(Range<usize>),
}

pub(super) trait LinebreaksExt {
    fn fragments<'a>(&'a mut self, s: &'a str) -> impl Iterator<Item = Fragment> + 'a;
}

impl LinebreaksExt for Linebreaks {
    fn fragments<'a>(&'a mut self, s: &'a str) -> impl Iterator<Item = Fragment> + 'a {
        let mut breaks = self.chunk(s);
        let mut start = 0;
        iter::from_fn(move || match breaks.next() {
            Some((index, opportunity)) => {
                let end = index - trailing_newline_len(&s[start..index]);
                let fragment = Fragment::Complete(start..end, opportunity);
                start = index;
                Some(fragment)
            }
            None if start < s.len() => {
                let end = s.len() - trailing_newline_len(&s[start..]);
                let fragment = Fragment::Partial(start..end);
                start = s.len();
                Some(fragment)
            }
            None => None,
        })
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
            fragments_str(input, &mut l).collect::<Vec<_>>(),
            &["foo", "bar", "baz"]
        );
    }

    fn fragments_str<'a: 'l, 'l>(
        s: &'a str,
        l: &'l mut Linebreaks,
    ) -> impl Iterator<Item = &'a str> + 'l {
        l.fragments(s).map(|f| match f {
            Complete(range, _) => &s[range],
            Partial(range) => &s[range],
        })
    }
}
