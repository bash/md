use crate::hyperlink::{CloseHyperlink, Hyperlink};
use anstyle::{Reset, Style};
use std::borrow::Cow;
use std::ops::Deref;
use std::{io, mem};
use textwrap::core::Fragment as TextwrapFragment;
use textwrap::wrap_algorithms::wrap_first_fit;
use url::Url;

/// A fragment of inline text that we can use with text wrapping.
#[derive(Debug)]
pub(crate) enum Fragment<'a> {
    Word(Word<'a>),
    SoftBreak,
    HardBreak,
    PushStyle(Style),
    PopStyle,
    SetLink(Url),
    UnsetLink,
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
    pub(crate) fn word(word: &str) -> Fragment<'_> {
        Fragment::Word(Word::new(word))
    }

    pub(crate) fn into_owned(self) -> Fragment<'static> {
        match self {
            Fragment::Word(w) => Fragment::Word(w.into_owned()),
            Fragment::SoftBreak => Fragment::SoftBreak,
            Fragment::HardBreak => Fragment::HardBreak,
            Fragment::PushStyle(s) => Fragment::PushStyle(s),
            Fragment::PopStyle => Fragment::PopStyle,
            Fragment::SetLink(url) => Fragment::SetLink(url),
            Fragment::UnsetLink => Fragment::UnsetLink,
        }
    }
}

impl TextwrapFragment for Fragment<'_> {
    fn width(&self) -> f64 {
        match self {
            Fragment::Word(w) => w.width(),
            Fragment::SoftBreak
            | Fragment::HardBreak
            | Fragment::PushStyle(_)
            | Fragment::PopStyle
            | Fragment::SetLink(_)
            | Fragment::UnsetLink => 0.,
        }
    }

    fn whitespace_width(&self) -> f64 {
        match self {
            Fragment::Word(w) => w.whitespace_width(),
            Fragment::SoftBreak => 1.,
            Fragment::PushStyle(_)
            | Fragment::PopStyle
            | Fragment::HardBreak
            | Fragment::SetLink(_)
            | Fragment::UnsetLink => 0.,
        }
    }

    fn penalty_width(&self) -> f64 {
        match self {
            Fragment::Word(w) => w.penalty_width(),
            Fragment::SoftBreak
            | Fragment::PushStyle(_)
            | Fragment::PopStyle
            | Fragment::HardBreak
            | Fragment::SetLink(_)
            | Fragment::UnsetLink => 0.,
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

    pub(crate) fn push_text(&mut self, text: &str) {
        self.extend(
            textwrap::WordSeparator::UnicodeBreakProperties
                .find_words(text)
                .map(|w| Fragment::Word(Word::from(w)).into_owned()),
        )
    }

    pub(crate) fn extend(&mut self, fragments: impl IntoIterator<Item = Fragment<'a>>) {
        self.fragments.extend(fragments);
    }
}

impl<'a> IntoIterator for Fragments<'a> {
    type Item = Fragment<'a>;
    type IntoIter = <Vec<Fragment<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fragments.into_iter()
    }
}

impl<'a> Deref for Fragments<'a> {
    type Target = [Fragment<'a>];

    fn deref(&self) -> &Self::Target {
        &self.fragments
    }
}

/// Writes a fragment and tracks the last used style.
///
/// This is useful when we want to render across multiple lines
/// when each line has a prefix with its own styling (e.g. blockquote, list).
#[derive(Debug)]
pub(crate) struct FragmentWriter {
    style_stack: StyleStack,
    link: Option<Url>,
    // The `id=` parameter helps the terminal connect links
    // that are broken up into multiple pieces (e.g. when we break the line)
    // https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
    link_id: usize,
}

impl FragmentWriter {
    pub fn new(default_style: Style) -> FragmentWriter {
        Self {
            style_stack: StyleStack::new(default_style),
            link: None,
            link_id: 0,
        }
    }
}

impl FragmentWriter {
    // TODO: trim trailing whitespace of each line
    pub(crate) fn write_block(
        &mut self,
        fragments: &[Fragment<'_>],
        available_columns: usize,
        w: &mut dyn io::Write,
        mut write_prefix: impl FnMut(&mut dyn io::Write) -> io::Result<()>,
    ) -> io::Result<()> {
        for forced_line in fragments.split(|f| matches!(f, Fragment::HardBreak)) {
            let lines = wrap_first_fit(forced_line, &[available_columns as f64]);
            for line in lines.into_iter() {
                if !line.is_empty() {
                    write_prefix(w)?;
                    write!(w, "{}", self.style_stack.head())?;
                    if let Some(url) = self.link.as_ref() {
                        write!(w, "{}", Hyperlink::new(url, self.link_id))?;
                    }
                    line.iter().try_for_each(|f| self.write(f, w))?;
                    write!(w, "{}", Reset)?;
                    if let Some(_) = self.link.as_ref() {
                        write!(w, "{CloseHyperlink}")?;
                    }
                    writeln!(w)?;
                }
            }
        }
        Ok(())
    }

    fn write(&mut self, f: &Fragment<'_>, w: &mut dyn io::Write) -> io::Result<()> {
        match f {
            Fragment::Word(word) => write!(w, "{}{}", word.word, word.whitespace),
            Fragment::SoftBreak => write!(w, " "),
            Fragment::HardBreak => unreachable!(),
            Fragment::PushStyle(s) => {
                self.style_stack.push(s.on_top_of(&self.style_stack.head()));
                write!(w, "{}", self.style_stack.head())
            }
            Fragment::PopStyle => {
                self.style_stack.pop();
                write!(w, "{}{}", Reset, self.style_stack.head())
            }
            Fragment::SetLink(url) => {
                if self.link.is_some() {
                    panic!("BUG: nested links"); // TODO: does pulldown-cmark support that?
                }
                self.link = Some(url.clone());
                self.link_id += 1;
                write!(w, "{}", Hyperlink::new(url, self.link_id))
            }
            Fragment::UnsetLink => {
                // This must handle the case where the link was not pushed
                // but is popped as not all `Tag::Link`s result in links but all `TagEnd::Link`s do.
                // Sending a "link reset" when no link is open is perfectly fine.
                self.link = None;
                write!(w, "{CloseHyperlink}")
            }
        }
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

trait StyleExt {
    fn on_top_of(&self, fallback: &Style) -> Style;
}

impl StyleExt for Style {
    fn on_top_of(&self, fallback: &Style) -> Style {
        Style::new()
            .effects(self.get_effects() | fallback.get_effects())
            .fg_color(self.get_fg_color().or_else(|| fallback.get_fg_color()))
            .bg_color(self.get_bg_color().or_else(|| fallback.get_bg_color()))
            .underline_color(
                self.get_underline_color()
                    .or_else(|| fallback.get_underline_color()),
            )
    }
}
