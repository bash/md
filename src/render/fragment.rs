use super::State;
use crate::fragment::{Fragment, FragmentWriter, WritePrefixFn};
use anstyle::{AnsiColor, Style};
use fmtastic::Superscript;
use pulldown_cmark::{Event, LinkType, Tag, TagEnd};
use std::io;
use url::Url;

// TODO: double spaces are usually not rendered in HTML, we should also filter that.
pub(super) trait FragmentWriterExt<'a> {
    fn try_write_event(&mut self, event: Event<'a>) -> io::Result<Option<Event<'a>>>;
}

impl<'a, F> FragmentWriterExt<'a> for FragmentWriter<'a, '_, F>
where
    F: WritePrefixFn,
{
    fn try_write_event(&mut self, event: Event<'a>) -> io::Result<Option<Event<'a>>> {
        // TODO: sort this match
        match event {
            Event::Text(t) => self.write(t)?,
            Event::Code(code) => {
                self.write(AnsiColor::Yellow.on_default().italic())?;
                self.write(code)?;
                self.write(Fragment::PopStyle)?;
            }
            Event::InlineMath(math) => {
                self.write(AnsiColor::Cyan.on_default().italic())?;
                self.write(math)?;
                self.write(Fragment::PopStyle)?;
            }
            Event::DisplayMath(math) => self.write_iter(display_math(&math))?,
            Event::Start(Tag::Strong) => self.write(Style::new().bold())?,
            Event::End(TagEnd::Strong) => self.write(Fragment::PopStyle)?,
            Event::Start(Tag::Emphasis) => self.write(Style::new().italic())?,
            Event::End(TagEnd::Emphasis) => self.write(Fragment::PopStyle)?,
            Event::Start(Tag::Strikethrough) => self.write(Style::new().strikethrough())?,
            Event::End(TagEnd::Strikethrough) => self.write(Fragment::PopStyle)?,
            Event::Start(Tag::Image { .. }) => {
                const NO_BREAK_SPACE: &str = "\u{00A0}";
                self.write(Style::new().invert())?;
                self.write("🖼")?;
                self.write(NO_BREAK_SPACE)?;
            }
            Event::End(TagEnd::Image) => {
                self.write(Fragment::PopStyle)?;
            }
            Event::SoftBreak => self.write(Fragment::SoftBreak)?,
            Event::HardBreak => self.write(Fragment::HardBreak)?,

            // TODO: get access to state somehow?
            // Event::Start(Tag::Link { .. }) if !state.options().hyperlinks => {}
            // Event::End(TagEnd::Link) if !state.options().hyperlinks => {}
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => link(self, link_type, &dest_url, &title, &id)?,
            Event::End(TagEnd::Link) => self.write(Fragment::UnsetLink)?,

            // Event::TaskListMarker is handled by the list item writer
            Event::InlineHtml(html) if is_br_tag(&html) => self.write(Fragment::HardBreak)?,

            Event::InlineHtml(_html) => {}
            // TODO: get access to state somehow?
            // Event::FootnoteReference(reference) => {
            //     self.write_iter(footnote_reference(&reference, state))?;
            // }
            event => return Ok(Some(event)),
        }

        Ok(None)
    }
}

fn is_br_tag(html: &str) -> bool {
    let html = html.replace(char::is_whitespace, "");
    html == "<br>" || html == "<br/>"
}

fn link<F>(
    f: &mut FragmentWriter<'_, '_, F>,
    _link_type: LinkType,
    dest_url: &str,
    _title: &str,
    _id: &str,
) -> io::Result<()>
where
    F: WritePrefixFn,
{
    // TODO: file links, test email
    if let Ok(url) = Url::parse(dest_url) {
        f.write(Fragment::SetLink(url))?;
    }
    Ok(())
}

fn footnote_reference<'b>(reference: &str, state: &mut State) -> [Fragment<'b>; 3] {
    let text = format!("{}", Superscript(state.get_footnote_number(reference)));
    [
        Fragment::PushStyle(AnsiColor::Green.on_default()),
        Fragment::Text(text.into()),
        Fragment::PopStyle,
    ]
}

fn display_math(_math: &str) -> [Fragment<'static>; 3] {
    // TODO: syntax highlight as latex?
    // TODO: allow writer to set/reset style
    [
        // Fragment::HardBreak,
        Fragment::PushStyle(AnsiColor::Red.on_default().invert()),
        Fragment::Text("[TODO: display math]".into()),
        Fragment::PopStyle,
        // Fragment::HardBreak,
    ]
}
