use std::fmt;

#[derive(Debug)]
pub(crate) struct Hyperlink<Url, Id> {
    url: Url,
    id: Option<Id>,
}

impl<Url, Id> Hyperlink<Url, Id> {
    pub(crate) fn new(url: Url, id: impl Into<Option<Id>>) -> Self {
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
pub(crate) struct CloseHyperlink;

impl fmt::Display for CloseHyperlink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{OSC}8;;{ST}")
    }
}
