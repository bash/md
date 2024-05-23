use crate::counting::SectionCounter;
use crate::display_width::DisplayWidth;
use crate::fmt_utils::NoDebug;
use crate::footnotes::FootnoteCounter;
use crate::fragment::{FragmentWriter, Fragments};
use crate::line_writer::LineWriter;
use crate::options::Options;
use anstyle::Style;
use std::borrow::Cow;
use std::io;
use std::ops::DerefMut as _;

type Prefix = DisplayWidth<Cow<'static, str>>;

#[derive(Debug)]
pub(super) struct Context<'a> {
    /// "Global" state used for the entirety of
    /// the rendering process.
    state: &'a mut State,
    /// Scoped state that can change (i.e. block quotes
    /// add a prefix).
    scope: Scope<'a>,
}

impl<'a> Context<'a> {
    pub(super) fn new(state: &'a mut State, output: &'a mut dyn io::Write) -> Self {
        Self {
            state,
            scope: Scope::new(output),
        }
    }

    pub(super) fn scope(
        &mut self,
        f: impl for<'s> FnOnce(&'s mut Scope<'a>) -> Scope<'s>,
    ) -> Context<'_> {
        Context {
            state: &mut self.state,
            scope: f(&mut self.scope),
        }
    }
}

impl<'a> Context<'a> {
    pub(super) fn available_columns(&self) -> usize {
        (self.state.options.columns as usize) - self.reserved_columns()
    }

    pub(super) fn write_fragments(&mut self, fragments: Fragments) -> io::Result<()> {
        let mut writer = FragmentWriter::new(self.style());
        writer.write_block(&fragments, self.available_columns(), &mut self.writer())
    }
}

#[derive(Debug)]
pub(super) struct State {
    options: Options,
    section_counter: SectionCounter,
    footnote_counter: FootnoteCounter,
}

impl State {
    pub(super) fn new(options: Options) -> Self {
        Self {
            options,
            section_counter: SectionCounter::default(),
            footnote_counter: FootnoteCounter::new(),
        }
    }
}

impl<'a> Context<'a> {
    pub(super) fn section_counter(&self) -> &SectionCounter {
        &self.state.section_counter
    }

    pub(super) fn get_footnote_number(&mut self, reference: &str) -> usize {
        self.state.footnote_counter.get_number(reference)
    }

    pub(super) fn section_counter_mut(&mut self) -> &mut SectionCounter {
        &mut self.state.section_counter
    }
}

#[derive(Debug)]
pub(super) struct Scope<'a> {
    output: NoDebug<&'a mut dyn io::Write>,
    first_block: bool,
    prefix: Prefix,
    style: Style,
}

impl<'a> Context<'a> {
    pub(super) fn style(&self) -> Style {
        self.scope.style
    }

    pub(super) fn writer(&mut self) -> LineWriter<'_> {
        LineWriter::new(self.scope.output.deref_mut(), self.scope.prefix.as_bytes())
    }

    pub(super) fn reserved_columns(&self) -> usize {
        self.scope.prefix.display_width()
    }

    pub(super) fn write_block_start(&mut self) -> io::Result<()> {
        if self.scope.first_block {
            self.scope.first_block = false;
            Ok(())
        } else {
            writeln!(self.scope.output, "{}", self.scope.prefix)
        }
    }
}

impl<'a> Scope<'a> {
    fn new(output: &'a mut dyn io::Write) -> Self {
        Self {
            output: NoDebug::from(output),
            first_block: true,
            prefix: DisplayWidth::from(Cow::Borrowed("")),
            style: Style::new(),
        }
    }

    pub(super) fn with_style<'b>(&'b mut self, style: Style) -> Scope<'b> {
        Scope {
            output: NoDebug::from(&mut self.output.0 as &mut dyn io::Write),
            first_block: true,
            prefix: DisplayWidth::from(self.prefix.value().clone()), // TODO: what does this clone?
            style,
        }
    }

    pub(super) fn with_prefix<'b>(&'b mut self, prefix: &str) -> Scope<'b> {
        Scope {
            output: NoDebug::from(&mut self.output.0 as &mut dyn io::Write),
            first_block: true,
            prefix: DisplayWidth::from(Cow::Owned(format!("{}{prefix}", self.prefix))),
            style: self.style,
        }
    }
}
