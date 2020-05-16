use cang_jie::{CangJieTokenizer, TokenizerOption};
use lindera_tantivy::tokenizer::LinderaTokenizer;
use tantivy::tokenizer::TokenizerManager;

pub use cang_jie::CANG_JIE as LANG_CN;
pub const LANG_JP: &'static str = "lindera";

pub fn tokenizers_register(tokenizers: &TokenizerManager) {
    tokenizers.register(
        LANG_CN,
        CangJieTokenizer {
            option: TokenizerOption::ForSearch { hmm: true },
            ..Default::default()
        },
    );
    tokenizers.register(LANG_JP, LinderaTokenizer::new("decompose", ""));
}
