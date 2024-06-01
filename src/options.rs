use std::marker::PhantomData;
use url::Url;

// TODO: Typst has a wonderful numbering system:
// https://github.com/typst/typst/blob/23746ee18901e08852306f35639298ad234d3481/crates/typst/src/model/numbering.rs
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
    /// Absolute URL that will be used as base for resolving
    /// relative links found in the document.
    pub base_url: Option<Url>,
    pub footnote_definition_placement: FootnoteDefinitionPlacement,
}

/// Where to place footnote definitions.
#[derive(Debug, Copy, Clone, Default)]
pub enum FootnoteDefinitionPlacement {
    /// Place all footnote definitions at the end of the document.
    #[default]
    EndOfDocument,
    /// Place the footnote definitions as they appear in the source.
    InPlace,
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
            base_url: None,
            footnote_definition_placement: FootnoteDefinitionPlacement::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
enum SymbolRepertoireImpl {
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
