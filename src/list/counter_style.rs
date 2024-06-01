use crate::context::Context;
use crate::prefix::Prefix;
use crate::style::StyledStr;
use anstyle::Style;
use CounterStyle::*;

#[derive(Debug)]
pub(super) enum CounterStyle<'a> {
    Numbered(u64),
    Bulleted(&'a str),
}

impl<'a> CounterStyle<'a> {
    pub(super) fn from_context(number: Option<u64>, ctx: &'a Context) -> Self {
        match number {
            Some(n) => Numbered(n),
            None => Bulleted(ctx.bullet()),
        }
    }

    pub(super) fn next(&mut self) {
        if let Numbered(n) = self {
            *n += 1;
        }
    }

    pub(super) fn to_prefix(&self) -> Prefix {
        let value = match self {
            Numbered(n) => format!("{n}. "),
            Bulleted(bullet) => format!("{bullet} "),
        };
        Prefix::continued(StyledStr::new(value, Style::new().bold()))
    }
}
