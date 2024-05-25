use pulldown_cmark::HeadingLevel;

#[derive(Debug, Default)]
pub(crate) struct SectionCounter {
    counters: [usize; 6],
    end: usize,
}

impl SectionCounter {
    pub(crate) fn update(&mut self, level: HeadingLevel) {
        let index = to_index(level);
        self.counters[index] += 1;
        self.end = index;
        for i in (index + 1)..self.counters.len() {
            self.counters[i] = 0;
        }
    }

    pub(crate) fn value(&self) -> &[usize] {
        &self.counters[0..=self.end]
    }
}

fn to_index(level: HeadingLevel) -> usize {
    (level as usize) - (HeadingLevel::H1 as usize)
}
