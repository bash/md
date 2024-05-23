use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct FootnoteCounter {
    numbers: HashMap<String, usize>,
}

impl FootnoteCounter {
    pub(crate) fn new() -> Self {
        Self {
            numbers: HashMap::default(),
        }
    }
}

impl FootnoteCounter {
    pub(crate) fn get_number(&mut self, reference: &str) -> usize {
        if let Some(number) = self.numbers.get(reference) {
            *number
        } else {
            let number = self.numbers.len() + 1;
            self.numbers.insert(reference.to_owned(), number);
            number
        }
    }
}
