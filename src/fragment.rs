use anstyle::{Reset, Style};
use std::borrow::Cow;
use std::{io, mem};
use textwrap::core::Fragment as TextwrapFragment;
use textwrap::wrap_algorithms::wrap_first_fit;

/// A fragment of inline text that we can use with text wrapping.
#[derive(Debug)]
pub(crate) enum Fragment<'a> {
    Word(Word<'a>),
    SoftBreak,
    PushStyle(Style),
    PopStyle,
}

impl<'a> From<Word<'a>> for Fragment<'a> {
    fn from(value: Word<'a>) -> Self {
        Fragment::Word(value)
    }
}

impl From<Style> for Fragment<'_> {
    fn from(value: Style) -> Self {
        Fragment::PushStyle(value)
    }
}

impl Fragment<'_> {
    pub(crate) fn from_str(line: &str) -> impl Iterator<Item = Fragment<'_>> {
        textwrap::WordSeparator::UnicodeBreakProperties
            .find_words(line)
            .map(|w| Fragment::Word(Word::from(w)))
    }

    pub(crate) fn into_owned(self) -> Fragment<'static> {
        match self {
            Fragment::Word(w) => Fragment::Word(w.into_owned()),
            Fragment::SoftBreak => Fragment::SoftBreak,
            Fragment::PushStyle(s) => Fragment::PushStyle(s),
            Fragment::PopStyle => Fragment::PopStyle,
        }
    }
}

impl TextwrapFragment for Fragment<'_> {
    fn width(&self) -> f64 {
        match self {
            Fragment::Word(w) => w.width(),
            Fragment::SoftBreak | Fragment::PushStyle(_) | Fragment::PopStyle => 0.,
        }
    }

    fn whitespace_width(&self) -> f64 {
        match self {
            Fragment::Word(w) => w.whitespace_width(),
            Fragment::SoftBreak => 1.,
            Fragment::PushStyle(_) | Fragment::PopStyle => 0.,
        }
    }

    fn penalty_width(&self) -> f64 {
        match self {
            Fragment::Word(w) => w.penalty_width(),
            Fragment::SoftBreak | Fragment::PushStyle(_) | Fragment::PopStyle => 0.,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Word<'a> {
    pub(crate) word: Cow<'a, str>,
    /// Whitespace to insert if the word does not fall at the end of a line.
    pub(crate) whitespace: Cow<'a, str>,
    /// Penalty string to insert if the word falls at the end of a line.
    pub(crate) penalty: Cow<'a, str>,
    // Cached width in columns.
    width: f64,
}

impl Word<'_> {
    pub(crate) fn new(word: &str) -> Word<'_> {
        Word::from(textwrap::core::Word::from(word))
    }

    pub(crate) fn into_owned(self) -> Word<'static> {
        Word {
            word: Cow::Owned(self.word.into_owned()),
            whitespace: Cow::Owned(self.whitespace.into_owned()),
            penalty: Cow::Owned(self.penalty.into_owned()),
            width: self.width,
        }
    }
}

impl<'a> From<textwrap::core::Word<'a>> for Word<'a> {
    fn from(w: textwrap::core::Word<'a>) -> Self {
        Self {
            word: Cow::Borrowed(w.word),
            whitespace: Cow::Borrowed(w.whitespace),
            penalty: Cow::Borrowed(w.penalty),
            width: textwrap::core::Fragment::width(&w),
        }
    }
}

impl TextwrapFragment for Word<'_> {
    fn width(&self) -> f64 {
        self.width
    }

    fn whitespace_width(&self) -> f64 {
        self.whitespace.len() as f64
    }

    fn penalty_width(&self) -> f64 {
        self.penalty.len() as f64
    }
}

#[derive(Debug, Default)]
pub(crate) struct Fragments<'a> {
    fragments: Vec<Fragment<'a>>,
}

impl<'a> Fragments<'a> {
    pub(crate) fn push(&mut self, fragment: impl Into<Fragment<'a>>) {
        self.fragments.push(fragment.into())
    }

    pub(crate) fn extend(&mut self, fragments: impl IntoIterator<Item = Fragment<'a>>) {
        self.fragments.extend(fragments);
    }

    pub(crate) fn take(&mut self) -> Vec<Fragment<'a>> {
        mem::replace(&mut self.fragments, Vec::new())
    }
}

/// Writes a fragment and tracks the last used style.
///
/// This is useful when we want to render across multiple lines
/// when each line has a prefix with its own styling (e.g. blockquote, list).
#[derive(Debug)]
pub(crate) struct FragmentWriter {
    style_stack: StyleStack,
}

impl FragmentWriter {
    pub fn new(default_style: Style) -> FragmentWriter {
        Self {
            style_stack: StyleStack::new(default_style),
        }
    }
}

impl FragmentWriter {
    pub(crate) fn write(&mut self, f: &Fragment<'_>, w: &mut dyn io::Write) -> io::Result<()> {
        match f {
            Fragment::Word(word) => write!(w, "{}{}", word.word, word.whitespace),
            Fragment::SoftBreak => write!(w, " "),
            Fragment::PushStyle(s) => {
                self.style_stack
                    .push(update_style(self.style_stack.head(), *s));
                write!(w, "{}", self.style_stack.head())
            }
            Fragment::PopStyle => {
                self.style_stack.pop();
                write!(w, "{}{}", Reset, self.style_stack.head())
            }
        }
    }

    pub(crate) fn write_block(
        &mut self,
        fragments: Vec<Fragment<'_>>,
        available_columns: usize,
        w: &mut dyn io::Write,
        mut write_prefix: impl FnMut(&mut dyn io::Write) -> io::Result<()>,
    ) -> io::Result<()> {
        let lines = wrap_first_fit(&fragments, &[available_columns as f64]);
        for line in lines.into_iter() {
            write!(w, "{}", Reset)?;
            write_prefix(w)?;
            write!(w, "{}", self.last_style())?;
            line.iter().try_for_each(|f| self.write(f, w))?;
            writeln!(w)?;
        }
        Ok(())
    }

    pub(crate) fn last_style(&self) -> Style {
        self.style_stack.head()
    }
}

#[derive(Debug)]
struct StyleStack {
    head: Style,
    tail: Vec<Style>,
}

impl StyleStack {
    fn new(head: Style) -> Self {
        Self {
            head,
            tail: Vec::default(),
        }
    }

    fn head(&self) -> Style {
        self.head
    }

    fn push(&mut self, style: Style) {
        self.tail.push(mem::replace(&mut self.head, style));
    }

    fn pop(&mut self) {
        // TODO: should we silently fail instead?
        self.head = self.tail.pop().expect("stack empty");
    }
}

fn update_style(old: Style, new: Style) -> Style {
    let combined = old | new.get_effects();
    combined
        .fg_color(old.get_fg_color().or_else(|| new.get_fg_color()))
        .bg_color(old.get_bg_color().or_else(|| new.get_bg_color()))
        .underline_color(
            old.get_underline_color()
                .or_else(|| new.get_underline_color()),
        )
}
