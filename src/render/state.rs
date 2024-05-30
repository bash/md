use super::writer::Writer;
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

    pub(super) fn available_columns(&self, w: &Writer) -> usize {
        (self.options.columns as usize) - w.reserved_columns()
    }

    pub(super) fn text_columns(&self, w: &Writer) -> usize {
        min(self.available_columns(w), self.options.text_max_columns)
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

    pub(super) fn bullet(&self, w: &Writer) -> &str {
        self.bullets.nth(w.nested_list_count())
    }
}
