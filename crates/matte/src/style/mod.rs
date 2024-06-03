use anstyle::Style;

mod stack;
pub(crate) use stack::*;
mod str;
pub(crate) use str::*;

pub(crate) trait StyleExt {
    fn on_top_of(&self, fallback: Style) -> Style;
}

impl StyleExt for Style {
    fn on_top_of(&self, fallback: Style) -> Style {
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
