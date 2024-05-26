use crate::options::SymbolRepertoire;

#[derive(Debug, Clone)]
pub(crate) struct Bullets(BulletsImpl);

impl Bullets {
    pub(crate) fn default_for(symbols: SymbolRepertoire) -> Self {
        if symbols.is_unicode() {
            Self(BulletsImpl::Static(&["•", "◦", "▪", "‣"]))
        } else {
            Self(BulletsImpl::Static(&["*", "-", "+"]))
        }
    }
}

impl Bullets {
    pub(crate) fn new(bullets: Vec<String>) -> Self {
        // TODO: panic if bullets is empty
        Self(BulletsImpl::Owned(bullets))
    }

    pub(crate) fn nth(&self, n: usize) -> &str {
        let index = n % self.len();
        match &self.0 {
            BulletsImpl::Owned(v) => &v[index],
            BulletsImpl::Static(s) => s[index],
        }
    }
}

impl Bullets {
    fn len(&self) -> usize {
        match &self.0 {
            BulletsImpl::Owned(v) => v.len(),
            BulletsImpl::Static(s) => s.len(),
        }
    }
}

#[derive(Debug, Clone)]
enum BulletsImpl {
    Owned(Vec<String>),
    Static(&'static [&'static str]),
}
