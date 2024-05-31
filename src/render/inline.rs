use super::State;
use crate::chars::NO_BREAK_SPACE;
use crate::inline::Inline;
use anstyle::{AnsiColor, Style};
use fmtastic::Superscript;
use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};
use smallvec::{smallvec, SmallVec};
use url::Url;

type Inlines<'a> = SmallVec<[Inline<'a>; 4]>;

macro_rules! inlines {
    ($($x:expr),*$(,)*) => {
        smallvec![$(Inline::from($x),)*]
    }
}

pub(super) fn into_inlines<'a>(event: Event<'a>, state: &mut State) -> Inlines<'a> {
    try_into_inlines(event, state).unwrap_or_else(|event| panic!("Unhandled event {event:#?}"))
}

// TODO: double spaces are usually not rendered in HTML, we should also filter that.
pub(super) fn try_into_inlines<'a>(
    event: Event<'a>,
    state: &mut State,
) -> Result<Inlines<'a>, Event<'a>> {
    match event {
        Event::Text(text) => Ok(inlines![text]),
        Event::Code(c) => Ok(code(c, AnsiColor::Yellow)),
        Event::InlineMath(math) => Ok(code(math, AnsiColor::Cyan)),
        Event::DisplayMath(_) => Ok(display_math()),
        Event::Start(Tag::Strong) => Ok(inlines![Style::new().bold()]),
        Event::End(TagEnd::Strong) => Ok(inlines![Inline::PopStyle]),
        Event::Start(Tag::Emphasis) => Ok(inlines![Style::new().italic()]),
        Event::End(TagEnd::Emphasis) => Ok(inlines![Inline::PopStyle]),
        Event::Start(Tag::Strikethrough) => Ok(inlines![Style::new().strikethrough()]),
        Event::End(TagEnd::Strikethrough) => Ok(inlines![Inline::PopStyle]),
        Event::Start(Tag::Image { .. }) => Ok(image_start()),
        Event::End(TagEnd::Image) => Ok(image_end()),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Ok(link(link_type, &dest_url, &title, &id, state)),
        Event::End(TagEnd::Link) => Ok(inlines![Inline::UnsetLink]),
        Event::SoftBreak => Ok(inlines![Inline::SoftBreak]),
        Event::HardBreak => Ok(inlines![Inline::HardBreak]),
        Event::InlineHtml(html) if is_br_tag(&html) => Ok(inlines![Inline::HardBreak]),
        Event::InlineHtml(_html) => Ok(Inlines::default()),
        Event::FootnoteReference(reference) => Ok(footnote_reference(&reference, state)),
        //
        // `Event::TaskListMarker` is handled by the list item writer so no need to handle it here.
        //
        // All other events are "rejected".
        event => Err(event),
    }
}

fn is_br_tag(html: &str) -> bool {
    let html = html.replace(char::is_whitespace, "");
    html == "<br>" || html == "<br/>"
}

fn code(code: CowStr, color: AnsiColor) -> Inlines {
    inlines![color.on_default().italic(), code, Inline::PopStyle]
}

fn display_math<'a>() -> Inlines<'a> {
    inlines![
        AnsiColor::Red.on_default().invert(),
        "[TODO: display math]",
        Inline::PopStyle,
    ]
}

fn image_start<'a>() -> Inlines<'a> {
    inlines![Style::new().invert(), "🖼", NO_BREAK_SPACE]
}

fn image_end<'a>() -> Inlines<'a> {
    inlines![Inline::PopStyle]
}

fn link<'a>(
    _link_type: LinkType,
    dest_url: &str,
    _title: &str,
    _id: &str,
    state: &State,
) -> Inlines<'a> {
    if state.options().hyperlinks {
        if let Some(url) = parse_url(dest_url, state) {
            return inlines![Inline::SetLink(url)];
        }
    }

    Inlines::default()
}

fn parse_url(url: &str, state: &State) -> Option<Url> {
    Url::parse(url).ok().or_else(|| {
        state
            .options()
            .base_url
            .as_ref()
            .and_then(|b| b.join(url).ok())
    })
}

fn footnote_reference<'a>(reference: &str, state: &mut State) -> Inlines<'a> {
    let text = format!("{}", Superscript(state.footnotes().get_number(reference)));
    inlines![
        AnsiColor::Green.on_default(),
        CowStr::from(text),
        Inline::PopStyle
    ]
}
