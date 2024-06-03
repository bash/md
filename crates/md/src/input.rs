use matte::file_uri::{current_dir, file_in_current_dir};
use matte::url::Url;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, stdin, StdinLock};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) enum InputArg {
    Stdin,
    Path(PathBuf),
}

impl InputArg {
    pub(crate) fn open(self) -> io::Result<InputReader> {
        match self {
            InputArg::Stdin => Ok(InputReader::Stdin(stdin().lock())),
            InputArg::Path(path) => Ok(InputReader::File(File::open(&path)?, path)),
        }
    }
}

impl<P: Into<PathBuf>> From<P> for InputArg {
    fn from(value: P) -> Self {
        let value = value.into();
        if value == Path::new("-") {
            InputArg::Stdin
        } else {
            InputArg::Path(value)
        }
    }
}

pub(crate) enum InputReader {
    Stdin(StdinLock<'static>),
    File(File, PathBuf),
}

impl InputReader {
    pub(crate) fn name(&self) -> &OsStr {
        match self {
            InputReader::Stdin(_) => OsStr::new("STDIN"),
            InputReader::File(_, path) => path.file_name().unwrap_or(path.as_os_str()),
        }
    }

    pub(crate) fn base_url(&self) -> Result<Url, matte::file_uri::Error> {
        match self {
            InputReader::Stdin(_) => current_dir(),
            InputReader::File(_, path) => file_in_current_dir(path),
        }
    }
}

impl Deref for InputReader {
    type Target = dyn io::Read;

    fn deref(&self) -> &Self::Target {
        match self {
            InputReader::Stdin(stdin) => stdin,
            InputReader::File(file, _) => file,
        }
    }
}

impl DerefMut for InputReader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            InputReader::Stdin(stdin) => stdin,
            InputReader::File(file, _) => file,
        }
    }
}
