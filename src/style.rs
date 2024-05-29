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
        // TODO: should we silently fail instead?
        self.head = self.tail.pop().expect("stack empty");
    }
}

pub(crate) trait StyleExt {
    fn on_top_of(&self, fallback: &Style) -> Style;
}

impl StyleExt for Style {
    fn on_top_of(&self, fallback: &Style) -> Style {
        Style::new()
            .effects(self.get_effects() | fallback.get_effects())
            .fg_color(self.get_fg_color().or_else(|| fallback.get_fg_color()))
            .bg_color(self.get_bg_color().or_else(|| fallback.get_bg_color()))
            .underline_color(
                self.get_underline_color()
                    .or_else(|| fallback.get_underline_color()),
            )
    }
}
