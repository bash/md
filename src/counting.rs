use pulldown_cmark::HeadingLevel;

#[derive(Debug, Default)]
pub(crate) struct SectionCounter {
    counters: [usize; 6],
}

impl SectionCounter {
    pub(crate) fn update(&mut self, level: HeadingLevel) {
        let index = to_index(level);
        self.counters[index] += 1;
        for i in (index + 1)..self.counters.len() {
            self.counters[i] = 0;
        }
    }

    pub(crate) fn value(&self) -> &[usize] {
        if let Some(first_zero_counter) = self.counters.iter().position(|c| *c == 0) {
            &self.counters[..first_zero_counter]
        } else {
            &self.counters
        }
    }
}

fn to_index(level: HeadingLevel) -> usize {
    (level as usize) - (HeadingLevel::H1 as usize)
}
