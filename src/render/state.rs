use crate::bullets::Bullets;
use crate::counting::SectionCounter;
use crate::fmt_utils::NoDebug;
use crate::footnotes::FootnoteCounter;
use crate::fragment::{FragmentWriter, Fragments};
use crate::options::Options;
use crate::prefix::{Prefix, PrefixMeasurement};
use anstyle::{Reset, Style};
use std::cmp::min;
use std::{io, iter, mem};

#[derive(Debug)]
pub(super) struct State<'a> {
    output: NoDebug<&'a mut dyn io::Write>,
    options: Options,
    section_counter: SectionCounter,
    footnote_counter: FootnoteCounter,
    bullets: Bullets,
    stack: Stack,
}

impl<'a> State<'a> {
    pub(super) fn new(output: &'a mut dyn io::Write, options: Options) -> Self {
        Self {
            output: NoDebug(output),
            options,
            section_counter: SectionCounter::default(),
            footnote_counter: FootnoteCounter::new(),
            bullets: Bullets::default(),
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
        // TODO: pass measurement by line
        writer.write_block(
            &fragments,
            min(self.available_columns(), self.options.text_max_columns),
            &mut *self.output,
            |w| write_prefix(&mut self.stack, w),
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

    pub(super) fn bullet(&self) -> &str {
        &mut self.bullets.nth(self.stack.head.nested_list_count)
    }
}

impl<'a> State<'a> {
    pub(super) fn style(&self) -> Style {
        self.stack.head.style
    }

    pub(super) fn write_prefix(&mut self) -> io::Result<()> {
        write_prefix(&mut self.stack, &mut *self.output)
    }

    pub(super) fn write_blank_line(&mut self) -> io::Result<()> {
        write_prefix(&mut self.stack, &mut *self.output)?;
        writeln!(self.output)
    }

    pub(super) fn writer(&mut self) -> &mut dyn io::Write {
        &mut *self.output
    }

    pub(super) fn reserved_columns(&self) -> usize {
        // TODO: caching
        self.stack
            .iter()
            .map(|b| b.prefix.measure())
            .sum::<PrefixMeasurement>()
            .first()
    }

    pub(super) fn write_block_start(&mut self) -> io::Result<()> {
        if self.stack.head.first_block {
            self.stack.head.first_block = false;
            Ok(())
        } else {
            self.write_blank_line()
        }
    }

    pub(super) fn unset_first_block(&mut self) {
        self.stack.head.first_block = false;
    }
}

fn write_prefix(stack: &mut Stack, w: &mut dyn io::Write) -> io::Result<()> {
    stack
        .iter_mut()
        .try_for_each(|b| write!(w, "{}{}{Reset}", b.style, b.prefix.take_next()))
}

impl<'a> State<'a> {
    pub(super) fn block<T>(
        &mut self,
        b: impl for<'r, 'b> FnOnce(&'r mut BlockBuilder<'b>) -> &'r mut BlockBuilder<'b>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let mut builder = BlockBuilder::for_state(&self);
        b(&mut builder);
        self.stack.push(builder.build());
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
        iter::once(&self.head).chain(self.tail.iter()).rev()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        iter::once(&mut self.head).chain(self.tail.iter_mut()).rev()
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
struct Block {
    first_block: bool,
    prefix: Prefix,
    style: Style,
    nested_list_count: usize,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            first_block: true,
            prefix: Default::default(),
            style: Default::default(),
            nested_list_count: 0,
        }
    }
}

#[derive(Debug)]
pub(super) struct BlockBuilder<'a> {
    parent: &'a Block,
    prefix: Prefix,
    style: Style,
    list: bool,
}

impl<'a> BlockBuilder<'a> {
    fn for_state(state: &'a State) -> Self {
        BlockBuilder {
            parent: &state.stack.head,
            list: false,
            prefix: Prefix::default(),
            style: state.stack.head.style,
        }
    }

    fn build(self) -> Block {
        Block {
            first_block: true,
            prefix: self.prefix,
            style: self.style,
            nested_list_count: if self.list {
                self.parent.nested_list_count + 1
            } else {
                self.parent.nested_list_count
            },
        }
    }
}

impl BlockBuilder<'_> {
    pub(super) fn styled(&mut self, f: impl FnOnce(Style) -> Style) -> &mut Self {
        self.style = f(self.parent.style);
        self
    }

    pub(super) fn prefix(&mut self, prefix: Prefix) -> &mut Self {
        self.prefix = prefix;
        self
    }

    pub(super) fn list(&mut self) -> &mut Self {
        self.list = true;
        self
    }
}
