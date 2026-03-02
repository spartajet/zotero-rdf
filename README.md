# zotero-rdf

A Rust library for parsing Zotero RDF/XML export files.

## Overview

`zotero-rdf` provides a simple and efficient way to parse Zotero library exports in RDF/XML format. It extracts structured bibliographic data including:

- Journal articles, books, conference papers, theses, and more
- Authors with proper ordering
- DOIs, abstracts, publication dates
- Attachments (PDFs, HTML snapshots, etc.)

## Features

- **Simple API**: Parse files with a single function call
- **Structured Output**: Get Rust structs with typed fields
- **Attachment Support**: Attachments are extracted as fields of parent items
- **Efficient**: Built on `oxrdf` for fast RDF parsing
- **Logging**: Integration with `tracing` for optional debug logging
- **Flexible**: Access both raw RDF graph and high-level structs

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zotero-rdf = "0.1"
```

## Quick Start

```rust
use zotero_rdf::{parse_file, Extractor};

// Parse a Zotero RDF export file
let graph = parse_file("my_library.rdf")?;

// Extract structured items
let extractor = Extractor::new(&graph);
let items = extractor.extract_all();

println!("Found {} items", items.len());

for item in items {
    if let Some(title) = &item.title {
        println!("Title: {}", title);
        println!("Authors: {}", item.authors.iter()
            .map(|a| a.display_name())
            .collect::<Vec<_>>()
            .join(", "));
    }
}
```

## Library Structure

### Basic Parsing

```rust
use zotero_rdf::parse_file;

// Simple parsing with default settings
let graph = parse_file("library.rdf")?;

// Access the RDF graph directly
println!("Graph contains {} triples", graph.len());
```

### Structured Extraction

```rust
use zotero_rdf::{parse_file, Extractor};

let graph = parse_file("library.rdf")?;
let extractor = Extractor::new(&graph);
let items = extractor.extract_all();

for item in items {
    println!("Type: {}", item.item_type);
    println!("Title: {:?}", item.title);
    println!("Authors: {:?}", item.authors);
    println!("Date: {:?}", item.date);
    println!("DOI: {:?}", item.doi);
    println!("Attachments: {}", item.attachments.len());
}
```

### Parse with Statistics

```rust
use zotero_rdf::parse_file_with_stats;

let (graph, stats) = parse_file_with_stats("library.rdf")?;

println!("Triples parsed: {}", stats.triples_count);
println!("Errors encountered: {}", stats.error_count);
```

### Custom Parse Options

```rust
use zotero_rdf::{parse_file_with_options, ParseOptions};

let options = ParseOptions {
    continue_on_error: true,
    max_errors: 100,
};

let graph = parse_file_with_options("library.rdf", options)?;
```

## Logging

The library uses the `tracing` crate for logging. To see what's happening during parsing:

```rust
use tracing_subscriber;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    // Set RUST_LOG=debug for more verbose output
    let graph = zotero_rdf::parse_file("library.rdf").unwrap();
}
```

Log levels:
- `INFO`: Key operations (file parsing, item extraction, completion stats)
- `DEBUG`: Detailed info (each item extracted, attachment counts)
- `TRACE`: Most verbose (each author, each attachment extraction)

## Data Model

### ZoteroItem

Represents a Zotero item (article, book, etc.):

```rust
pub struct ZoteroItem {
    pub uri: String,              // Unique identifier
    pub item_type: String,         // e.g., "journalArticle", "book"
    pub title: Option<String>,
    pub authors: Vec<Author>,
    pub date: Option<String>,
    pub doi: Option<String>,
    pub abstract_note: Option<String>,
    pub attachments: Vec<Attachment>,
}
```

### Author

```rust
pub struct Author {
    pub given_name: Option<String>,  // First name
    pub surname: Option<String>,     // Last name
    pub full_name: Option<String>,   // Full name if available
}

impl Author {
    pub fn display_name(&self) -> String {
        // Returns "Surname, GivenName" format
    }
}
```

### Attachment

```rust
pub struct Attachment {
    pub uri: String,
    pub title: Option<String>,       // Usually filename
    pub content_type: Option<String>, // MIME type (e.g., "application/pdf")
    pub url: Option<String>,         // File path or URL
}
```

## Item Types

Zotero supports many item types. Common ones include:

- `journalArticle` - Journal articles
- `book` - Books
- `conferencePaper` - Conference papers
- `thesis` - Theses and dissertations
- `report` - Reports
- `webpage` - Web pages
- `document` - Generic documents

## Error Handling

```rust
use zotero_rdf::{parse_file, ZoteroRdfError};

match parse_file("library.rdf") {
    Ok(graph) => {
        println!("Parsed successfully!");
    }
    Err(ZoteroRdfError::Io { .. }) => {
        eprintln!("Failed to read file");
    }
    Err(ZoteroRdfError::ParseError { .. }) => {
        eprintln!("Failed to parse RDF/XML");
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Performance

The library is designed for efficiency:

- Parsing: ~5000 triples in ~100ms
- Extraction: ~100 items in ~25ms
- Memory: ~1MB per 2000 triples

*Benchmarks on typical hardware. Results may vary.*

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please feel free to submit a Pull Request.

## See Also

- [Zotero](https://www.zotero.org/) - The reference manager
- [oxrdf](https://docs.rs/oxrdf) - The underlying RDF library
- [RDF/XML Syntax](https://www.w3.org/TR/rdf-syntax-grammar/) - Specification

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
