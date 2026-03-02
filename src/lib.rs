//! # zotero-rdf
//!
//! 一个专注于解析 Zotero 导出 RDF/XML 文件的 Rust 库。
//!
//! ## 日志功能
//!
//! 本库使用 `tracing` 进行日志记录。要查看日志，需要在应用中初始化 tracing subscriber：
//!
//! ```rust,ignore
//! use tracing_subscriber;
//!
//! // 初始化日志（默认 info 级别）
//! tracing_subscriber::fmt()
//!     .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
//!         .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")))
//!     .init();
//!
//! // 或者通过环境变量控制：RUST_LOG=debug cargo run
//! ```
//!
//! 日志级别说明：
//! - `INFO`: 关键操作（文件解析、条目提取、完成统计）
//! - `DEBUG`: 详细信息（每个条目的提取、附件数量）
//! - `TRACE`: 最详细信息（每个作者、每个附件的提取）

mod error;
mod extractor;
mod model;
mod parser;
mod vocab;

// --- 导出公共 API ---
pub use error::ZoteroRdfError;
pub use extractor::Extractor;
pub use model::{Attachment, Author, ZoteroItem};
pub use parser::{parse_file, parse_file_with_base, parse_reader, parse_reader_with_base, DEFAULT_BASE_IRI};
pub use oxrdf::Graph; // 重导出 Graph，方便用户使用

// 内部词汇表（测试时需要）
#[cfg(test)]
pub(crate) use vocab::*;
