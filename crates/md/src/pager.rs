use std::borrow::Cow;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::{env, fmt, io};

#[derive(Debug, Clone)]
pub(crate) struct Pager {
    program: Option<Cow<'static, str>>,
    args: Vec<String>,
    kind: PagerKind,
}

impl Pager {
    pub(crate) fn from_env() -> Option<Self> {
        Self::from_env_var("MD_PAGER").or_else(|| Self::from_env_var("PAGER"))
    }

    pub(crate) fn less_from_env() -> Self {
        let args = if env::var_os("LESS").is_some() {
            // If the user has set `LESS` then they want to customize
            // the args passed to `less`. Let's not override any preferences.
            Vec::new()
        } else {
            vec!["--quit-if-one-screen".to_owned()]
        };
        Self {
            program: Some(Cow::Borrowed("less")),
            args,
            kind: PagerKind::Less,
        }
    }

    pub(crate) fn hyperlinks(&self) -> bool {
        use PagerKind::*;
        !matches!(self.kind, More | Most)
    }

    // Takes a best-effort guess at how much
    // potential decorations of the pager take up.
    pub(crate) fn decoration_width(&self) -> usize {
        match self.kind {
            PagerKind::Bat => 10,
            _ => 0,
        }
    }

    fn from_env_var(name: &str) -> Option<Self> {
        let value = env::var(name).ok()?;
        let mut words = shell_words::split(&value).ok()?;
        let mut words = words.drain(..);
        let program = words.next().map(Cow::Owned);
        let kind = program
            .as_deref()
            .map(PagerKind::from_program)
            .unwrap_or(PagerKind::Other);
        let args = words.collect();
        Some(Pager {
            program,
            args,
            kind,
        })
    }
}

impl Pager {
    /// Spawns a pager with an optional title (supported by less and bat).
    pub(crate) fn spawn(&self, title: &str) -> io::Result<Option<(Child, ChildStdin)>> {
        // An empty `PAGER` env var disables paging.
        // No need to try to spawn a process in that case.
        let Some(program) = self.program.as_deref() else {
            return Ok(None);
        };

        let mut command = Command::new(program);

        command
            .stdin(Stdio::piped())
            .args(&self.args)
            .args(self.mandatory_args())
            .args(self.title_args(title))
            // Setting it as env var means that we get a nice prompt
            // when using bat (which in turn uses less) as pager.
            .env("LESS", less_prompt_env_var(title));

        match command.spawn() {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
            Ok(mut child) => {
                let stdin = child.stdin.take().expect("stdin is always piped");
                Ok(Some((child, stdin)))
            }
        }
    }

    fn mandatory_args(&self) -> &'static [&'static str] {
        match self.kind {
            PagerKind::Less => &["--RAW-CONTROL-CHARS"],
            PagerKind::Bat => &["--language", "txt"],
            _ => &[],
        }
    }

    fn title_args(&self, title: &str) -> Vec<String> {
        match self.kind {
            PagerKind::Bat => vec!["--file-name".to_owned(), title.to_owned()],
            _ => Vec::default(),
        }
    }
}

fn less_prompt_env_var(title: &str) -> OsString {
    let mut args = env::var_os("LESS").unwrap_or_default();
    if !args.is_empty() {
        args.push(" ");
    }
    // From less' man page:
    // > Some options [...] require a string to follow the option letter.
    // > The string for that option is considered to end when a dollar sign (`$`) is found.
    _ = write!(args, "-Ps{}$", LessPrompt { title });
    args
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PagerKind {
    Less,
    More,
    Most,
    Bat,
    Other,
}

impl PagerKind {
    fn from_program(program: &str) -> Self {
        match Path::new(program).file_stem().and_then(|s| s.to_str()) {
            Some("less") => PagerKind::Less,
            Some("more") => PagerKind::More,
            Some("most") => PagerKind::Most,
            Some("bat" | "batcat") => PagerKind::Bat,
            Some(_) | None => PagerKind::Other,
        }
    }
}

// The less prompt is heavily inspired by `man`:
// https://gitlab.com/man-db/man-db/-/blob/main/src/man.c
struct LessPrompt<'a> {
    title: &'a str,
}

impl fmt::Display for LessPrompt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ?ltline %lt?L/%L.:byte %bB?s/%s..?e (END):?pB %pB\\%.. (press h for help or q to quit)",
            LessEscape(self.title)
        )
    }
}

/// From less' man page:
/// > Any characters other than the special ones (question mark, colon, period, percent, and backslash)
/// > become literally part of the prompt. Any of the special characters may be included in
/// > the prompt literally by preceding it with a backslash.
///
/// Why is the dollar replaced? \
/// Again, quoting from less' man page:
/// > Some options [...] require a string to follow the option letter.
/// > The string for that option is considered to end when a dollar sign (`$`) is found.
struct LessEscape<'a>(&'a str);

impl fmt::Display for LessEscape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.chars() {
            let c = if c == '$' { '?' } else { c };
            if matches!(c, '?' | ':' | '.' | '%' | '\\') {
                write!(f, "\\")?;
            }
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}
