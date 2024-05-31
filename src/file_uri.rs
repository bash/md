use std::ffi::OsString;
use std::path::Path;
use std::{env, error, fmt, io};
use url::Url;

/// Gets a file URI pointing to the current directory.
pub fn current_dir() -> Result<Url, Error> {
    let current_dir = env::current_dir()?;
    let mut url = Url::from_directory_path(current_dir).expect("current dir is absolute");
    set_current_host(&mut url)?;
    return Ok(url);
}

pub fn file_in_current_dir(file_path: impl AsRef<Path>) -> Result<Url, Error> {
    let file_path = file_path.as_ref();
    let mut url = if file_path.is_absolute() {
        Url::from_file_path(file_path).expect("file path is absolute")
    } else {
        let mut p = env::current_dir()?;
        p.push(file_path);
        Url::from_file_path(p).expect("current dir is absolute")
    };
    set_current_host(&mut url)?;
    return Ok(url);
}

// Explicitly setting the hostname allows
// terminal emulators some freedome (e.g. when connected over SSH).
// See: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
fn set_current_host(url: &mut Url) -> Result<(), Error> {
    let host = hostname::get()?;
    let Some(host_str) = host.to_str() else {
        return Err(Error::Unicode(host));
    };
    url.set_host(Some(host_str))?;
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(url::ParseError),
    Unicode(OsString),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::Parse(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Parse(e) => write!(f, "URL parse error: {e}"),
            Error::Unicode(data) => {
                write!(f, "Invalid unicode data: {}", Path::new(data).display())
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e.source().unwrap_or(e)),
            Error::Parse(e) => Some(e.source().unwrap_or(e)),
            Error::Unicode(_) => None,
        }
    }
}
