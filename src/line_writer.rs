use memchr::memchr;
use std::error::Error;
use std::{fmt, io};

/// An adapter over [`io::Write`] that prefixes every line.
pub(crate) struct LineWriter<'a> {
    inner: &'a mut dyn io::Write,
    prefix: &'a [u8],
    state: State,
}

enum State {
    NewLine,
    Prefix(usize),
    InLine,
}

impl<'a> LineWriter<'a> {
    pub(crate) fn new(inner: &'a mut dyn io::Write, prefix: &'a [u8]) -> Self {
        Self {
            inner,
            prefix,
            state: State::NewLine,
        }
    }

    pub(crate) fn raw(&mut self) -> &mut dyn io::Write {
        &mut self.inner
    }
}

impl<'a> io::Write for LineWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.state {
            State::NewLine => {
                self.state = State::Prefix(self.inner.write(&self.prefix)?);
                interrupt()
            }
            State::Prefix(bytes_written) => {
                if bytes_written == self.prefix.len() {
                    self.state = State::InLine;
                    interrupt()
                } else {
                    self.state = State::Prefix(bytes_written + self.inner.write(&self.prefix)?);
                    interrupt()
                }
            }
            State::InLine => {
                if let Some(line_ending) = memchr(b'\n', buf) {
                    let bytes_written = self.inner.write(&buf[0..=line_ending])?;
                    if bytes_written == line_ending + 1 {
                        self.state = State::NewLine;
                        Ok(bytes_written)
                    } else {
                        Ok(bytes_written)
                    }
                } else {
                    self.inner.write(buf)
                }
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn interrupt<T>() -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::Interrupted, Empty))
}

#[derive(Debug)]
struct Empty;

impl Error for Empty {}

impl fmt::Display for Empty {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
