use anstyle::Style;
use std::mem;

#[derive(Debug)]
pub(crate) struct StyleStack {
    head: Style,
    tail: Vec<Style>,
}

impl StyleStack {
    pub(crate) fn new(head: Style) -> Self {
        Self {
            head,
            tail: Vec::default(),
        }
    }

    pub(crate) fn head(&self) -> Style {
        self.head
    }

    pub(crate) fn push(&mut self, style: Style) {
        self.tail.push(mem::replace(&mut self.head, style));
    }

    pub(crate) fn pop(&mut self) {
        if let Some(new_head) = self.tail.pop() {
            self.head = new_head;
        } else if cfg!(debug_assertions) {
            panic!("stack empty");
        }
    }
}
