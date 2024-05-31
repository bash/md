use super::context::BlockContext;
use crate::bullets::Bullets;
use crate::counting::SectionCounter;
use crate::footnotes::FootnoteCounter;
use crate::options::Options;
use std::cmp::min;

#[derive(Debug)]
pub(super) struct State {
    options: Options,
    section_counter: SectionCounter,
    footnote_counter: FootnoteCounter,
    bullets: Bullets,
}

impl State {
    pub(super) fn new(options: Options) -> Self {
        Self {
            bullets: Bullets::default_for(options.symbol_repertoire),
            options,
            section_counter: SectionCounter::default(),
            footnote_counter: FootnoteCounter::new(),
        }
    }
}

impl State {
    pub(super) fn options(&self) -> &Options {
        &self.options
    }

    // TODO: rename to `available_width`
    pub(super) fn available_columns(&self, b: &BlockContext) -> usize {
        (self.options.columns as usize) - b.prefix_width()
    }

    pub(super) fn text_columns(&self, b: &BlockContext) -> usize {
        min(self.available_columns(b), self.options.text_max_columns)
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

    pub(super) fn bullet(&self, b: &BlockContext) -> &str {
        self.bullets.nth(b.list_depth())
    }
}
