use super::State;
use crate::fragment::{Fragment, Fragments, Word};
use anstyle::{AnsiColor, Style};
use fmtastic::Superscript;
use pulldown_cmark::{Event, Tag, TagEnd};

pub(super) trait FragmentsExt {
    fn try_push_event<'a>(&mut self, event: Event<'a>, state: &mut State) -> Option<Event<'a>>;
}

impl FragmentsExt for Fragments<'_> {
    fn try_push_event<'a>(&mut self, event: Event<'a>, state: &mut State) -> Option<Event<'a>> {
        // TODO: sort this match
        match event {
            Event::Text(t) => self.push_text(&t),
            Event::Code(code) => {
                self.push(AnsiColor::Yellow.on_default().italic());
                self.push_text(&code);
                self.push(Fragment::PopStyle);
            }
            Event::InlineMath(math) => {
                self.push(AnsiColor::Cyan.on_default().italic());
                self.push_text(&math);
                self.push(Fragment::PopStyle);
            }
            Event::DisplayMath(math) => self.extend(display_math(&math)),
            Event::Start(Tag::Strong) => self.push(Style::new().bold()),
            Event::End(TagEnd::Strong) => self.push(Fragment::PopStyle),
            Event::Start(Tag::Emphasis) => self.push(Style::new().italic()),
            Event::End(TagEnd::Emphasis) => self.push(Fragment::PopStyle),
            Event::Start(Tag::Strikethrough) => self.push(Style::new().strikethrough()),
            Event::End(TagEnd::Strikethrough) => self.push(Fragment::PopStyle),
            Event::Start(Tag::Image { .. }) => {
                self.push(Style::new().invert());
                self.push(Word::new("🖼 "));
            }
            Event::End(TagEnd::Image) => {
                self.push(Fragment::PopStyle);
            }
            Event::SoftBreak => self.push(Fragment::SoftBreak),
            Event::HardBreak => self.push(Fragment::HardBreak),
            Event::Start(Tag::Link { .. }) => {} // TODO: links
            Event::End(TagEnd::Link) => {}
            Event::TaskListMarker(checked) => self.push(task_list_marker(checked)),
            Event::InlineHtml(_html) => {}
            Event::FootnoteReference(reference) => {
                self.extend(footnote_reference(&reference, state))
            }
            _ => return Some(event),
        }

        None
    }
}

fn footnote_reference<'b>(reference: &str, state: &mut State) -> [Fragment<'b>; 3] {
    let text = format!("{}", Superscript(state.get_footnote_number(reference)));
    [
        Fragment::PushStyle(AnsiColor::Green.on_default()),
        Fragment::word(&text).into_owned(),
        Fragment::PopStyle,
    ]
}

fn task_list_marker(checked: bool) -> Fragment<'static> {
    if checked {
        Fragment::word("☒ ")
    } else {
        Fragment::word("☐ ")
    }
}

fn display_math(_math: &str) -> [Fragment<'static>; 5] {
    // TODO: syntax highlight as latex?
    // TODO: allow writer to set/reset style
    [
        Fragment::HardBreak,
        Fragment::PushStyle(AnsiColor::Red.on_default().invert()),
        Fragment::word("[TODO: display math]").into_owned(),
        Fragment::PopStyle,
        Fragment::HardBreak,
    ]
}
