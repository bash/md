use std::fmt;

/// Writes a hyperlink to the terminal using the `OSC 8` sequence.
///
/// Docs: <https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda>
#[derive(Debug)]
pub(super) struct Hyperlink<Url, Id> {
    url: Url,
    // The `id=` parameter helps the terminal connect links
    // that are broken up into multiple pieces (e.g. when we break the line)
    id: Option<Id>,
}

impl<Url, Id> Hyperlink<Url, Id> {
    pub(super) fn new(url: Url, id: impl Into<Option<Id>>) -> Self {
        Self { url, id: id.into() }
    }
}

const OSC: &str = "\x1b]";
const ST: &str = "\x1b\\";

impl<Url, Id> fmt::Display for Hyperlink<Url, Id>
where
    Url: fmt::Display,
    Id: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{OSC}8;")?;
        if let Some(id) = &self.id {
            write!(f, "id={}", id)?;
        }
        write!(f, ";{}{ST}", self.url)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct CloseHyperlink;

impl fmt::Display for CloseHyperlink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{OSC}8;;{ST}")
    }
}
