use crate::counting::SectionCounter;
use crate::prefix::Prefix;
use pulldown_cmark::HeadingLevel;
use std::borrow::Cow;
use std::fmt::Write as _;
use std::str::FromStr;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum HeadingDecoration {
    None,
    #[default]
    Numbered,
    Prefixed(Cow<'static, str>),
}

impl FromStr for HeadingDecoration {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "numbering" {
            Ok(HeadingDecoration::Numbered)
        } else if let Some(prefix) = s
            .strip_prefix("prefixed(")
            .and_then(|s| s.strip_suffix(')'))
        {
            Ok(HeadingDecoration::Prefixed(Cow::Owned(prefix.to_owned())))
        } else {
            Err(())
        }
    }
}

impl HeadingDecoration {
    pub(super) fn prefix(&self, level: HeadingLevel, c: impl FnOnce() -> SectionCounter) -> Prefix {
        use HeadingDecoration::*;
        match self {
            Numbered => Prefix::continued(numbering(c())),
            Prefixed(prefix) => Prefix::continued(prefix.repeat(level as usize)),
            None => Prefix::default(),
        }
    }
}

// TODO: having numbering for changelog files is really not nice
// Since this needs to be configurable anyways maybe we can have a heuristic
// that detects changelog files by name (any or no extension):
// * changelog, CHANGELOG, RELEASE_NOTES, releasenotes, RELEASENOTES
// others?
fn numbering(counters: SectionCounter) -> String {
    let mut output = String::new();
    let numbers = &counters.as_slice()[1..];

    // No numbering for sections with leading zeroes.
    if !numbers.is_empty() && !numbers.starts_with(&[0]) {
        for n in numbers {
            write!(output, "{n}.").unwrap(); // TODO
        }
        write!(output, " ").unwrap(); // TODO
    }

    output
}
