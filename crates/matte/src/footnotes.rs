use pulldown_cmark::Event;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Default)]
pub(crate) struct Footnotes<'e>(RefCell<FootnotesData<'e>>);

#[derive(Debug, Default)]
struct FootnotesData<'e> {
    indexes: HashMap<String, usize>,
    footnotes: Vec<Footnote<'e>>,
}

#[derive(Debug)]
pub(crate) struct Footnote<'e> {
    pub(crate) number: usize,
    pub(crate) events: Vec<Event<'e>>,
}

impl<'e> Footnotes<'e> {
    pub(crate) fn get_number(&self, reference: &str) -> usize {
        self.0.borrow_mut().get_index(reference) + 1
    }

    pub(crate) fn push(&self, reference: &str, event: Event<'e>) {
        self.0.borrow_mut().push(reference, event)
    }

    pub(crate) fn take(&self) -> Vec<Footnote<'e>> {
        mem::take(&mut self.0.borrow_mut().footnotes)
    }
}

impl<'e> FootnotesData<'e> {
    fn get_index(&mut self, reference: &str) -> usize {
        if let Some(index) = self.indexes.get(reference) {
            *index
        } else {
            let index = self.footnotes.len();
            self.indexes.insert(reference.to_owned(), index);
            self.footnotes.push(Footnote {
                events: Vec::default(),
                number: index + 1,
            });
            index
        }
    }

    // TODO: improve this as it currently needs a lookup for every push
    fn push(&mut self, reference: &str, event: Event<'e>) {
        let number = self.get_index(reference);
        let footnote = &mut self.footnotes[number];
        footnote.events.push(event);
    }
}
