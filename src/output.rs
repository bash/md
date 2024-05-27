use core::fmt;
use std::io::{self, stdout, LineWriter, Stdout, Write as _};
use std::process::{Child, ChildStdin};
use thiserror::Error;

use crate::pager::Pager;
use crate::paging::PagingChoice;

#[derive(Debug)]
pub(crate) enum Output {
    Stdout(Stdout),
    Pager(Pager, Child, Option<LineWriter<ChildStdin>>),
}

impl Output {
    pub(crate) fn from_env(title: &str, paging: PagingChoice) -> io::Result<Self> {
        if let Some(paged) = Self::paged_from_env(title, paging)? {
            write!(stdout(), "{}", SetTitle(title))?;
            stdout().flush()?;
            Ok(paged)
        } else {
            Ok(Output::Stdout(stdout()))
        }
    }

    fn paged_from_env(title: &str, paging: PagingChoice) -> io::Result<Option<Self>> {
        if paging.should_enable() {
            let pager = Pager::from_env().unwrap_or_else(|| Pager::less_from_env());
            let Some((child, stdin)) = pager.spawn(title)? else {
                return Ok(None);
            };
            Ok(Some(Output::Pager(
                pager,
                child,
                Some(LineWriter::new(stdin)),
            )))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn hyperlinks(&self) -> bool {
        match self {
            Output::Stdout(_) => true,
            Output::Pager(pager, _, _) => pager.hyperlinks(),
        }
    }

    pub(crate) fn decoration_width(&self) -> usize {
        match self {
            Output::Stdout(_) => 0,
            Output::Pager(pager, _, _) => pager.decoration_width(),
        }
    }
}

macro_rules! deref_output {
    ($self:expr) => {{
        let writer: &mut dyn std::io::Write = match $self {
            Output::Stdout(stdout) => stdout,
            Output::Pager(_pager, _child, Some(writer)) => writer,
            Output::Pager(_, _, None) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    PagerClosed,
                ))
            }
        };
        writer
    }};
}

impl io::Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        deref_output!(self).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        deref_output!(self).flush()
    }
}

#[derive(Debug, Error)]
#[error("Pager closed")]
struct PagerClosed;

impl Drop for Output {
    fn drop(&mut self) {
        if let Output::Pager(_, child, stdin) = self {
            stdin.take(); // Consume the stdin so that the pager knows we're done.
            _ = child.wait();
        }
    }
}

struct SetTitle<'a>(&'a str);

impl fmt::Display for SetTitle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const OSC: &str = "\x1b]";
        const ST: &str = "\x1b\\";
        write!(f, "{OSC}0;md {title}{ST}", title = self.0)
    }
}
