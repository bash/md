use std::iter;
use unicode_linebreak_chunked::{BreakOpportunity, Linebreaks};

// A piece of text that doesn't contain
// any break opportunities.
#[derive(Debug)]
pub(super) enum Fragment {
    Complete(usize, usize, BreakOpportunity),
    Partial(usize),
}

pub(super) trait LinebreaksExt {
    fn fragments<'a>(&'a mut self, s: &'a str) -> impl Iterator<Item = Fragment> + 'a;
}

impl LinebreaksExt for Linebreaks {
    fn fragments<'a>(&'a mut self, s: &'a str) -> impl Iterator<Item = Fragment> + 'a {
        let mut breaks = self.chunk(&s);
        let mut start = 0;
        iter::from_fn(move || match breaks.next() {
            Some((index, opportunity)) => {
                let fragment = Fragment::Complete(start, index, opportunity);
                start = index;
                Some(fragment)
            }
            None if start < s.len() => {
                let fragment = Fragment::Partial(start);
                start = s.len();
                Some(fragment)
            }
            None => None,
        })
    }
}
