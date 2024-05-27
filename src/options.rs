use std::marker::PhantomData;

#[derive(Debug)]
#[non_exhaustive]
pub struct Options {
    // TODO: use u64 for integer types that are not indexes.
    pub columns: u16,
    pub text_max_columns: usize,
    pub symbol_repertoire: SymbolRepertoire,
    // pub rule_style: RuleStyle,
    // pub show_metadata_blocks: bool,
    pub hyperlinks: bool,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct SymbolRepertoire(SymbolRepertoireImpl);

impl SymbolRepertoire {
    pub const fn unicode(emoji: bool) -> Self {
        if emoji {
            SymbolRepertoire(SymbolRepertoireImpl::UnicodeWithEmoji)
        } else {
            SymbolRepertoire(SymbolRepertoireImpl::Unicode)
        }
    }

    pub const fn ascii() -> Self {
        SymbolRepertoire(SymbolRepertoireImpl::Ascii)
    }

    pub(crate) fn is_unicode(self) -> bool {
        matches!(
            self.0,
            SymbolRepertoireImpl::Unicode | SymbolRepertoireImpl::UnicodeWithEmoji
        )
    }

    pub(crate) fn has_emoji(self) -> bool {
        matches!(self.0, SymbolRepertoireImpl::UnicodeWithEmoji)
    }
}

impl Options {
    pub fn plain_text(columns: u16) -> Self {
        Self {
            columns,
            text_max_columns: 100,
            symbol_repertoire: SymbolRepertoire::unicode(true),
            hyperlinks: true,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum SymbolRepertoireImpl {
    #[default]
    UnicodeWithEmoji,
    Unicode,
    Ascii,
}

#[derive(Debug)]
pub struct RuleStyle(PhantomData<()>);

impl RuleStyle {
    /// A horizonal line.
    pub const fn line() -> Self {
        todo!()
    }

    /// Three spaced asterisks: "∗ ∗ ∗".
    pub const fn dinkus() -> Self {
        todo!()
    }

    /// A fleuron: "❧".
    pub const fn fleuron() -> Self {
        todo!()
    }

    /// An ornamental symbol e.g. a dingbat or a fleuron.
    pub const fn ornament(c: char) -> Self {
        todo!()
    }
}
