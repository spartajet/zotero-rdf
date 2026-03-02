//! # zotero-rdf
//!
//! A Rust library for parsing Zotero RDF/XML export files.
//!
//! ## Overview
//!
//! This library provides a simple and efficient way to parse Zotero library exports
//! in RDF/XML format. It extracts structured bibliographic data including authors,
//! DOIs, abstracts, and attachments.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use zotero_rdf::{parse_file, Extractor};
//!
//! // Parse a Zotero RDF export file
//! let graph = parse_file("my_library.rdf")?;
//!
//! // Extract structured items
//! let extractor = Extractor::new(&graph);
//! let items = extractor.extract_all();
//!
//! for item in items {
//!     println!("Title: {:?}", item.title);
//!     println!("Authors: {}", item.authors.iter()
//!         .map(|a| a.display_name())
//!         .collect::<Vec<_>>()
//!         .join(", "));
//! }
//! # Ok::<(), zotero_rdf::ZoteroRdfError>(())
//! ```
//!
//! ## Logging
//!
//! The library uses `tracing` for structured logging. To see what's happening during parsing:
//!
//! ```rust,ignore
//! use tracing_subscriber;
//!
//! // Initialize logging (default: info level)
//! tracing_subscriber::fmt()
//!     .with_env_filter(
//!         tracing_subscriber::EnvFilter::from_default_env()
//!             .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
//!     )
//!     .init();
//!
//! // Set RUST_LOG=debug for more verbose output
//! ```
//!
//! ### Log Levels
//!
//! - `INFO`: Key operations (file parsing, item extraction, completion stats)
//! - `DEBUG`: Detailed info (each item extracted, attachment counts)
//! - `TRACE`: Most verbose (each author, each attachment extraction)

mod error;
mod extractor;
mod model;
mod parser;
mod vocab;

// --- Public API exports ---
pub use error::{ErrorLocation, ParseOptions, ParseStats, ZoteroRdfError};
pub use extractor::Extractor;
pub use model::{Attachment, Author, ZoteroItem};
pub use oxrdf::Graph;
pub use parser::{
    DEFAULT_BASE_IRI, parse_file, parse_file_with_base, parse_file_with_options,
    parse_file_with_options_and_base, parse_file_with_stats, parse_reader, parse_reader_with_base,
    parse_reader_with_options, parse_reader_with_stats,
};
