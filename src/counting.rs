use std::cell::RefCell;

use pulldown_cmark::HeadingLevel;

#[derive(Debug, Default)]
pub(crate) struct Counters {
    section: RefCell<SectionCounter>,
}

impl Counters {
    pub(crate) fn update_section(&self, level: HeadingLevel) {
        self.section.borrow_mut().update(level)
    }

    pub(crate) fn section(&self) -> SectionCounter {
        self.section.borrow().clone()
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SectionCounter {
    counters: [usize; 6],
    end: usize,
}

impl SectionCounter {
    pub(crate) fn as_slice(&self) -> &[usize] {
        &self.counters[0..=self.end]
    }

    fn update(&mut self, level: HeadingLevel) {
        let index = to_index(level);
        self.counters[index] += 1;
        self.end = index;
        for i in (index + 1)..self.counters.len() {
            self.counters[i] = 0;
        }
    }
}

fn to_index(level: HeadingLevel) -> usize {
    (level as usize) - (HeadingLevel::H1 as usize)
}
