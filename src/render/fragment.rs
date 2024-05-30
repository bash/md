use super::State;
use crate::fragment::Fragment;
use anstyle::{AnsiColor, Style};
use fmtastic::Superscript;
use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};
use smallvec::{smallvec, SmallVec};
use url::Url;

type Fragments<'a> = SmallVec<[Fragment<'a>; 4]>;

macro_rules! fragments {
    ($($x:expr),*$(,)*) => {
        smallvec![$(Fragment::from($x),)*]
    }
}

pub(super) fn into_fragments<'a>(event: Event<'a>, state: &mut State) -> Fragments<'a> {
    try_into_fragments(event, state).unwrap_or_else(|event| panic!("Unhandled event {event:#?}"))
}

// TODO: double spaces are usually not rendered in HTML, we should also filter that.
pub(super) fn try_into_fragments<'a>(
    event: Event<'a>,
    state: &mut State,
) -> Result<Fragments<'a>, Event<'a>> {
    match event {
        Event::Text(text) => Ok(fragments![text]),
        Event::Code(c) => Ok(code(c)),
        Event::InlineMath(math) => Ok(code(math)),
        Event::DisplayMath(_) => Ok(display_math()),
        Event::Start(Tag::Strong) => Ok(fragments![Style::new().bold()]),
        Event::End(TagEnd::Strong) => Ok(fragments![Fragment::PopStyle]),
        Event::Start(Tag::Emphasis) => Ok(fragments![Style::new().italic()]),
        Event::End(TagEnd::Emphasis) => Ok(fragments![Fragment::PopStyle]),
        Event::Start(Tag::Strikethrough) => Ok(fragments![Style::new().strikethrough()]),
        Event::End(TagEnd::Strikethrough) => Ok(fragments![Fragment::PopStyle]),
        Event::Start(Tag::Image { .. }) => Ok(image_start()),
        Event::End(TagEnd::Image) => Ok(image_end()),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Ok(link(link_type, &dest_url, &title, &id, state)),
        Event::End(TagEnd::Link) => Ok(fragments![Fragment::UnsetLink]),
        Event::SoftBreak => Ok(fragments![Fragment::SoftBreak]),
        Event::HardBreak => Ok(fragments![Fragment::HardBreak]),
        Event::InlineHtml(html) if is_br_tag(&html) => Ok(fragments![Fragment::HardBreak]),
        Event::InlineHtml(_html) => Ok(Fragments::default()),
        Event::FootnoteReference(reference) => Ok(footnote_reference(&reference, state)),
        // `Event::TaskListMarker` is handled by the list item writer so no need to handle it here.
        // All other events are "rejected".
        event => Err(event),
    }
}

fn is_br_tag(html: &str) -> bool {
    let html = html.replace(char::is_whitespace, "");
    html == "<br>" || html == "<br/>"
}

fn code(code: CowStr) -> Fragments {
    fragments![
        AnsiColor::Cyan.on_default().italic(),
        code,
        Fragment::PopStyle
    ]
}

fn display_math<'a>() -> Fragments<'a> {
    fragments![
        AnsiColor::Red.on_default().invert(),
        "[TODO: display math]",
        Fragment::PopStyle,
    ]
}

fn image_start<'a>() -> Fragments<'a> {
    const NO_BREAK_SPACE: &str = "\u{00A0}";
    fragments![Style::new().invert(), "🖼", NO_BREAK_SPACE]
}

fn image_end<'a>() -> Fragments<'a> {
    fragments![Fragment::PopStyle]
}

fn link<'a>(
    _link_type: LinkType,
    dest_url: &str,
    _title: &str,
    _id: &str,
    state: &State,
) -> Fragments<'a> {
    if state.options().hyperlinks {
        // TODO: file links, test email
        if let Ok(url) = Url::parse(dest_url) {
            return fragments![Fragment::SetLink(url)];
        }
    }

    Fragments::default()
}

fn footnote_reference<'a>(reference: &str, state: &mut State) -> Fragments<'a> {
    let text = format!("{}", Superscript(state.get_footnote_number(reference)));
    fragments![
        AnsiColor::Green.on_default(),
        CowStr::from(text),
        Fragment::PopStyle
    ]
}
