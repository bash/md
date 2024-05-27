use super::State;
use crate::fragment::{Fragment, Fragments, Word};
use anstyle::{AnsiColor, Style};
use fmtastic::Superscript;
use pulldown_cmark::{Event, LinkType, Tag, TagEnd};
use url::Url;

// TODO: double spaces are usually not rendered in HTML, we should also filter that.

pub(super) trait FragmentsExt {
    fn try_push_event<'a>(&mut self, event: &Event<'a>, state: &mut State) -> bool;
}

impl FragmentsExt for Fragments<'_> {
    fn try_push_event<'a>(&mut self, event: &Event<'a>, state: &mut State) -> bool {
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

            Event::Start(Tag::Link { .. }) if !state.options().hyperlinks => {}
            Event::End(TagEnd::Link) if !state.options().hyperlinks => {}
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => link(self, *link_type, &dest_url, &title, &id),
            Event::End(TagEnd::Link) => self.push(Fragment::UnsetLink),

            // Event::TaskListMarker is handled by the list item writer
            Event::InlineHtml(_html) => {}
            Event::FootnoteReference(reference) => {
                self.extend(footnote_reference(&reference, state))
            }
            _ => return false,
        }

        true
    }
}

fn link(f: &mut Fragments, _link_type: LinkType, dest_url: &str, _title: &str, _id: &str) {
    // TODO: file links, test email
    if let Ok(url) = Url::parse(dest_url) {
        f.push(Fragment::SetLink(url));
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
