use super::context::BlockContext;
use crate::bullets::Bullets;
use crate::counting::SectionCounter;
use crate::footnotes::Footnotes;
use crate::options::Options;
use std::cmp::min;
use unicode_width::UnicodeWidthStr as _;

#[derive(Debug)]
pub(super) struct State<'e> {
    options: Options,
    section_counter: SectionCounter,
    footnotes: Footnotes<'e>,
    bullets: Bullets,
}

impl State<'_> {
    pub(super) fn new(options: Options) -> Self {
        Self {
            bullets: Bullets::default_for(options.symbol_repertoire),
            options,
            section_counter: SectionCounter::default(),
            footnotes: Footnotes::default(),
        }
    }
}

impl<'e> State<'e> {
    pub(super) fn options(&self) -> &Options {
        &self.options
    }

    pub(super) fn footnotes(&self) -> &Footnotes<'e> {
        &self.footnotes
    }

    // TODO: rename to `available_width`
    pub(super) fn available_columns(&self, b: &BlockContext) -> usize {
        (self.options.columns as usize) - b.prefix_chain().width()
    }

    pub(super) fn text_columns(&self, b: &BlockContext) -> usize {
        min(self.available_columns(b), self.options.text_max_columns)
    }

    pub(super) fn section_counter(&self) -> &SectionCounter {
        &self.section_counter
    }

    pub(super) fn section_counter_mut(&mut self) -> &mut SectionCounter {
        &mut self.section_counter
    }

    pub(super) fn bullet(&self, b: &BlockContext) -> &str {
        self.bullets.nth(b.list_depth())
    }
}
