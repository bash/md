use super::state::State;
use crate::fmt_utils::NoDebug;
use crate::fragment::{FragmentWriter, WritePrefixFn};
use crate::prefix::Prefix;
use anstyle::{Reset, Style};
use std::{io, iter, mem};
use unicode_width::UnicodeWidthStr as _;

#[derive(Debug)]
pub(super) struct Writer<'w> {
    output: NoDebug<&'w mut dyn io::Write>,
    // TODO: move stack to state
    stack: Stack,
}

impl<'w> Writer<'w> {
    pub(super) fn new(output: &'w mut dyn io::Write) -> Self {
        Self {
            output: NoDebug(output),
            stack: Stack::default(),
        }
    }

    pub(super) fn block<T>(
        &mut self,
        b: impl for<'r, 'b> FnOnce(&'r mut BlockBuilder<'b>) -> &'r mut BlockBuilder<'b>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let mut builder = BlockBuilder::new(self);
        b(&mut builder);
        self.stack.push(builder.build());
        let result = f(self);
        self.stack.pop();
        result
    }

    pub(super) fn style(&self) -> Style {
        self.stack.head.style
    }

    pub(super) fn nested_list_count(&self) -> usize {
        self.stack.head.nested_list_count
    }

    pub(super) fn write_prefix(&mut self) -> io::Result<()> {
        write_prefix(&mut self.stack, &mut *self.output)
    }

    pub(super) fn write_blank_line(&mut self) -> io::Result<()> {
        write_prefix(&mut self.stack, &mut *self.output)?;
        writeln!(self.output)
    }

    pub(super) fn fragment_writer<'i, 's>(
        &'s mut self,
        state: &State,
    ) -> FragmentWriter<'i, 's, impl WritePrefixFn + 's> {
        FragmentWriter::new(
            self.style(),
            state.text_columns(self),
            &mut *self.output,
            |w| write_prefix(&mut self.stack, w),
        )
    }

    pub(super) fn reserved_columns(&self) -> usize {
        // TODO: caching
        self.stack.iter().map(|b| b.prefix.width()).sum::<usize>()
    }

    pub(super) fn write_block_start(&mut self) -> io::Result<()> {
        if self.stack.head.first_block {
            self.stack.head.first_block = false;
            Ok(())
        } else {
            self.write_blank_line()
        }
    }
}

impl io::Write for Writer<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.output.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        self.output.write_fmt(fmt)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.output.write_vectored(bufs)
    }
}

fn write_prefix(stack: &mut Stack, w: &mut dyn io::Write) -> io::Result<()> {
    stack
        .iter_mut()
        .try_for_each(|b| write!(w, "{}{}{Reset}", b.style, b.prefix.take_next()))
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
    fn new(state: &'a Writer) -> Self {
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
