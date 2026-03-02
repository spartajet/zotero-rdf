mod error;
mod parser;
mod vocab;

// --- 导出公共 API ---
pub use error::ZoteroRdfError;
pub use parser::{parse_file, parse_file_with_base, parse_reader, parse_reader_with_base, DEFAULT_BASE_IRI};
pub use oxrdf::Graph; // 重导出 Graph，方便用户使用

// 内部词汇表（测试时需要）
#[cfg(test)]
pub(crate) use vocab::*;
