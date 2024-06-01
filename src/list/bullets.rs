use crate::options::SymbolRepertoire;

#[derive(Debug, Clone)]
pub(crate) struct Bullets(&'static [&'static str]);

impl Bullets {
    pub(crate) fn default_for(symbols: SymbolRepertoire) -> Self {
        if symbols.is_unicode() {
            Self::new(&["•", "◦", "▪", "‣"])
        } else {
            Self::new(&["*", "-", "+"])
        }
    }

    pub(crate) fn nth(&self, n: usize) -> &str {
        self.0[n % self.0.len()]
    }

    const fn new(bullets: &'static [&'static str]) -> Self {
        if bullets.is_empty() {
            panic!("bullets must not be an empty list")
        }

        Self(bullets)
    }
}
