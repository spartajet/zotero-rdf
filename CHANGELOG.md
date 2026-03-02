# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of zotero-rdf library
- Parse Zotero RDF/XML export files
- Extract structured bibliographic data (ZoteroItem)
- Extract author information with proper ordering
- Extract attachments (PDFs, HTML snapshots, etc.)
- Extract DOI, abstract, publication date
- Support for multiple item types (journalArticle, book, thesis, etc.)
- Parse statistics (triples count, error count)
- Configurable parse options (strict/lenient mode)
- Structured logging with tracing
- Comprehensive error handling with location information

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Basic RDF/XML parsing functionality
- Structured item extraction
- Author ordering support via rdf:Seq
- Attachment extraction as item fields
- DOI extraction from bibo:doi or URI
- Multi-language character support (UTF-8)
- Performance benchmarks
- Encoding tests for CJK characters

[Unreleased]: https://github.com/yourusername/zotero-rdf/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/zotero-rdf/releases/tag/v0.1.0
