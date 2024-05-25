use crate::counting::SectionCounter;
use crate::display_width::DisplayWidth;
use crate::fmt_utils::NoDebug;
use crate::footnotes::FootnoteCounter;
use crate::fragment::{FragmentWriter, Fragments};
use crate::options::Options;
use anstyle::Style;
use std::borrow::Cow;
use std::{io, iter, mem};

type Prefix = DisplayWidth<Cow<'static, str>>;

#[derive(Debug)]
pub(super) struct State<'a> {
    output: NoDebug<&'a mut dyn io::Write>,
    options: Options,
    section_counter: SectionCounter,
    footnote_counter: FootnoteCounter,
    stack: Stack,
}

impl<'a> State<'a> {
    pub(super) fn new(output: &'a mut dyn io::Write, options: Options) -> Self {
        Self {
            output: NoDebug(output),
            options,
            section_counter: SectionCounter::default(),
            footnote_counter: FootnoteCounter::new(),
            stack: Stack::default(),
        }
    }
}

impl<'a> State<'a> {
    pub(super) fn available_columns(&self) -> usize {
        (self.options.columns as usize) - self.reserved_columns()
    }

    pub(super) fn write_fragments(&mut self, fragments: Fragments) -> io::Result<()> {
        let mut writer = FragmentWriter::new(self.style());
        writer.write_block(
            &fragments,
            self.available_columns(),
            &mut *self.output,
            |w| write_prefix(&self.stack, w),
        )
    }
}

impl<'a> State<'a> {
    pub(super) fn section_counter(&self) -> &SectionCounter {
        &self.section_counter
    }

    pub(super) fn get_footnote_number(&mut self, reference: &str) -> usize {
        self.footnote_counter.get_number(reference)
    }

    pub(super) fn section_counter_mut(&mut self) -> &mut SectionCounter {
        &mut self.section_counter
    }
}

impl<'a> State<'a> {
    pub(super) fn style(&self) -> Style {
        self.stack.head.style
    }

    pub(super) fn write_prefix(&mut self) -> io::Result<()> {
        write_prefix(&self.stack, &mut *self.output)
    }

    pub(super) fn writer(&mut self) -> &mut dyn io::Write {
        &mut *self.output
    }

    pub(super) fn reserved_columns(&self) -> usize {
        // TODO: caching
        self.stack.iter().map(|b| b.prefix.display_width()).sum()
    }

    pub(super) fn write_block_start(&mut self) -> io::Result<()> {
        if self.stack.head.first_block {
            self.stack.head.first_block = false;
            Ok(())
        } else {
            self.write_prefix()?;
            writeln!(self.output)
        }
    }
}

fn write_prefix(stack: &Stack, w: &mut dyn io::Write) -> io::Result<()> {
    stack.iter().try_for_each(|b| write!(w, "{}", b.prefix))
}

impl<'a> State<'a> {
    pub(super) fn block<T>(&mut self, block: Block, f: impl FnOnce(&mut Self) -> T) -> T {
        self.stack.push(block);
        let result = f(self);
        self.stack.pop();
        result
    }
}

#[derive(Debug, Default)]
struct Stack {
    head: Block,
    tail: Vec<Block>,
}

impl Stack {
    fn iter(&self) -> impl Iterator<Item = &Block> {
        iter::once(&self.head).chain(self.tail.iter())
    }

    fn push(&mut self, block: Block) {
        let old_head = mem::replace(&mut self.head, block);
        self.tail.push(old_head);
    }

    fn pop(&mut self) {
        self.head = self.tail.pop().expect("stack empty");
    }
}

#[derive(Debug)]
pub(super) struct Block {
    first_block: bool,
    prefix: Prefix,
    style: Style,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            first_block: true,
            prefix: Default::default(),
            style: Default::default(),
        }
    }
}

impl Block {
    pub(super) fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub(super) fn with_prefix(mut self, prefix: impl Into<Cow<'static, str>>) -> Self {
        self.prefix = DisplayWidth::from(prefix.into());
        self
    }
}
