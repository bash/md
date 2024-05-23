use crate::counting::SectionCounter;
use crate::display_width::DisplayWidth;
use crate::fmt_utils::NoDebug;
use crate::footnotes::FootnoteCounter;
use crate::fragment::{FragmentWriter, Fragments};
use crate::line_writer::LineWriter;
use crate::options::Options;
use anstyle::Style;
use std::borrow::Cow;
use std::ops::DerefMut;
use std::{io, mem};

type Prefix = DisplayWidth<Cow<'static, str>>;

#[derive(Debug)]
pub(super) struct RenderState<'a> {
    output: NoDebug<&'a mut dyn io::Write>,
    options: Options,
    first_block: bool,
    section_counter: SectionCounter,
    footnote_counter: FootnoteCounter,
    prefix: Prefix,
    style: Style,
}

impl<'a> RenderState<'a> {
    pub(super) fn new(output: &'a mut dyn io::Write, options: Options) -> Self {
        Self {
            output: NoDebug::from(output),
            options,
            first_block: true,
            section_counter: SectionCounter::default(),
            footnote_counter: FootnoteCounter::new(),
            prefix: DisplayWidth::from(Cow::Borrowed("")),
            style: Style::new(),
        }
    }
}

impl RenderState<'_> {
    pub(super) fn writer(&mut self) -> LineWriter<'_> {
        LineWriter::new(self.output.deref_mut(), self.prefix.as_bytes())
    }

    pub(super) fn available_columns(&self) -> usize {
        (self.options.columns as usize) - self.prefix.display_width()
    }

    pub(super) fn style(&self) -> Style {
        self.style
    }

    pub(super) fn section_counter(&self) -> &SectionCounter {
        &self.section_counter
    }

    pub(super) fn get_footnote_number(&mut self, reference: &str) -> usize {
        self.footnote_counter.get_number(reference)
    }

    pub(super) fn section_counter_mut(&mut self) -> &mut SectionCounter {
        &mut self.section_counter
    }
}

impl RenderState<'_> {
    pub(super) fn write_block_start(&mut self) -> io::Result<()> {
        if self.first_block {
            self.first_block = false;
            Ok(())
        } else {
            writeln!(self.output, "{}", self.prefix)
        }
    }
}

impl RenderState<'_> {
    pub(super) fn write_fragments(&mut self, fragments: Fragments, style: Style) -> io::Result<()> {
        let mut writer = FragmentWriter::new(style);
        writer.write_block(&fragments, self.available_columns(), &mut self.writer())
    }
}

impl RenderState<'_> {
    pub(super) fn scope(&mut self, style: Style, prefix: Option<&'static str>) -> Scope {
        let prefix = self.push_prefix(prefix);
        let style = mem::replace(&mut self.style, style);
        let first_block = mem::replace(&mut self.first_block, true);
        Scope {
            backup: Backup {
                first_block,
                style,
                prefix,
            },
        }
    }

    pub(super) fn end_scope(&mut self, scope: Scope) {
        self.first_block = scope.backup.first_block;
        self.style = scope.backup.style;
        if let Some(prefix) = scope.backup.prefix {
            self.prefix = prefix;
        }
    }

    fn push_prefix(&mut self, prefix: Option<&'static str>) -> Option<Prefix> {
        if let Some(prefix) = prefix {
            let combined = DisplayWidth::from(Cow::Owned(format!("{}{}", &self.prefix, prefix)));
            Some(mem::replace(&mut self.prefix, combined))
        } else {
            None
        }
    }
}

#[must_use]
pub(super) struct Scope {
    backup: Backup,
}

struct Backup {
    first_block: bool,
    style: Style,
    prefix: Option<Prefix>,
}
