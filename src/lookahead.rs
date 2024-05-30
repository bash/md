use std::collections::VecDeque;
use std::mem;

pub(crate) struct Lookaheadable<T, I> {
    buffer: VecDeque<Option<T>>,
    inner: I,
}

impl<T, I> Lookaheadable<T, I> {
    pub(crate) fn new(inner: I) -> Self {
        Lookaheadable {
            inner,
            buffer: VecDeque::new(),
        }
    }

    pub(crate) fn lookahead(&mut self) -> Lookahead<'_, T, I> {
        Lookahead {
            inner: self,
            buffer: VecDeque::new(),
        }
    }
}

impl<T, I> Iterator for Lookaheadable<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(buffered) = self.buffer.pop_front() {
            buffered
        } else {
            self.inner.next()
        }
    }
}

impl<T, I> Lookaheadable<T, I> {}

pub(crate) struct Lookahead<'a, T, I> {
    buffer: VecDeque<Option<T>>,
    inner: &'a mut Lookaheadable<T, I>,
}

impl<'a, T, I> Iterator for Lookahead<'a, T, I>
where
    I: Iterator<Item = T>,
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        self.buffer.push_back(item.clone());
        item
    }
}

impl<'a, T, I> Lookahead<'a, T, I> {
    pub(crate) fn commit(mut self) -> impl Iterator<Item = T> {
        let buffer = mem::take(&mut self.buffer);
        buffer.into_iter().flatten()
    }
}

impl<'a, T, I> Drop for Lookahead<'a, T, I> {
    fn drop(&mut self) {
        for item in self.buffer.drain(..).rev() {
            self.inner.buffer.push_front(item)
        }
    }
}
