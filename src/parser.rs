use crate::error::{ParseOptions, ParseStats, ZoteroRdfError};
use oxrdf::Graph;
use oxrdfxml::RdfXmlParser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::{debug, info, instrument, warn};

/// Default base IRI for resolving relative IRIs in Zotero export files
///
/// Zotero RDF exports typically use relative URIs (e.g., `#item_123`).
/// This base IRI is used to resolve these relative references.
pub const DEFAULT_BASE_IRI: &str = "http://zotero.org/export#";

/// Parses a Zotero RDF file from a file path into an in-memory graph
///
/// # Arguments
///
/// * `path` - Path to the RDF file, can be relative or absolute
///
/// # Returns
///
/// Returns `oxrdf::Graph` on success, `ZoteroRdfError` on failure
///
/// # Example
///
/// ```rust,no_run
/// use zotero_rdf::parse_file;
///
/// let graph = parse_file("my_library.rdf")?;
/// println!("Loaded {} triples", graph.len());
/// # Ok::<(), zotero_rdf::ZoteroRdfError>(())
/// ```
///
/// # Errors
///
/// This function may return the following errors:
/// - `ZoteroRdfError::Io` - File does not exist or cannot be read
/// - `ZoteroRdfError::ParseError` - RDF/XML format error
/// - `ZoteroRdfError::InvalidUri` - Invalid URI format in the file
#[instrument(skip(path), fields(path = %path.as_ref().display()))]
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Graph, ZoteroRdfError> {
    parse_file_with_options(path, ParseOptions::default())
}

/// Parses a Zotero RDF file from a file path with a specified base IRI
///
/// # Arguments
///
/// * `path` - Path to the RDF file
/// * `base_iri` - Base IRI for resolving relative URIs
///
/// # Example
///
/// ```rust,no_run
/// use zotero_rdf::parse_file_with_base;
///
/// let graph = parse_file_with_base("my_library.rdf", "http://example.org/base#")?;
/// # Ok::<(), zotero_rdf::ZoteroRdfError>(())
/// ```
#[instrument(skip(path), fields(path = %path.as_ref().display(), base_iri = %base_iri))]
pub fn parse_file_with_base<P: AsRef<Path>>(
    path: P,
    base_iri: &str,
) -> Result<Graph, ZoteroRdfError> {
    parse_file_with_options_and_base(path, base_iri, ParseOptions::default())
}

/// Parses a file with custom options
///
/// # Arguments
///
/// * `path` - Path to the RDF file
/// * `options` - Parse options
///
/// # Example
///
/// ```rust,no_run
/// use zotero_rdf::{parse_file_with_options, ParseOptions};
///
/// let options = ParseOptions::lenient(); // Lenient mode
/// let graph = parse_file_with_options("my_library.rdf", options)?;
/// # Ok::<(), zotero_rdf::ZoteroRdfError>(())
/// ```
#[instrument(skip(path), fields(path = %path.as_ref().display()))]
pub fn parse_file_with_options<P: AsRef<Path>>(
    path: P,
    options: ParseOptions,
) -> Result<Graph, ZoteroRdfError> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    parse_reader_with_options(reader, DEFAULT_BASE_IRI, options)
}

/// Parses a file with custom options and base IRI
#[instrument(skip(path), fields(path = %path.as_ref().display(), base_iri = %base_iri))]
pub fn parse_file_with_options_and_base<P: AsRef<Path>>(
    path: P,
    base_iri: &str,
    options: ParseOptions,
) -> Result<Graph, ZoteroRdfError> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    parse_reader_with_options(reader, base_iri, options)
}

/// Parses RDF from any Reader (core logic)
///
/// Uses the default base IRI (`http://zotero.org/export#`)
pub fn parse_reader<R: Read>(reader: R) -> Result<Graph, ZoteroRdfError> {
    parse_reader_with_options(reader, DEFAULT_BASE_IRI, ParseOptions::default())
}

/// Parses RDF from any Reader with a specified base IRI
pub fn parse_reader_with_base<R: Read>(
    reader: R,
    base_iri: &str,
) -> Result<Graph, ZoteroRdfError> {
    parse_reader_with_options(reader, base_iri, ParseOptions::default())
}

/// Parses RDF from a Reader with custom options
#[instrument(skip(reader), fields(base_iri = %base_iri))]
pub fn parse_reader_with_options<R: Read>(
    reader: R,
    base_iri: &str,
    options: ParseOptions,
) -> Result<Graph, ZoteroRdfError> {
    let mut graph = Graph::default();

    // Use oxrdfxml parser with base IRI for resolving relative IRIs
    let parser = RdfXmlParser::new()
        .with_base_iri(base_iri)
        .map_err(|e| ZoteroRdfError::InvalidUri { uri: e.to_string() })?;

    // for_reader returns a Triple iterator
    let mut stats = ParseStats::default();

    for triple_result in parser.for_reader(reader) {
        match triple_result {
            Ok(triple) => {
                graph.insert(triple.as_ref());
                stats.triples_count += 1;
            }
            Err(e) => {
                stats.error_count += 1;
                warn!("Failed to parse triple: {}", e);

                if !options.continue_on_error || stats.error_count >= options.max_errors {
                    return Err(ZoteroRdfError::parse_error(format!(
                        "Too many parse errors ({}), stopping. Last error: {}",
                        stats.error_count, e
                    )));
                }
            }
        }
    }

    info!(
        "RDF parsing complete: {} triples, {} errors",
        stats.triples_count, stats.error_count
    );
    debug!("Graph contains {} triples", graph.len());

    Ok(graph)
}

/// Parses a file and returns detailed statistics
///
/// # Returns
///
/// Returns a tuple `(Graph, ParseStats)` containing the parse result and statistics
///
/// # Example
///
/// ```rust,no_run
/// use zotero_rdf::parse_file_with_stats;
///
/// let (graph, stats) = parse_file_with_stats("my_library.rdf")?;
/// println!("Parsed {} triples with {} errors",
///     stats.triples_count, stats.error_count);
/// # Ok::<(), zotero_rdf::ZoteroRdfError>(())
/// ```
#[instrument(skip(path), fields(path = %path.as_ref().display()))]
pub fn parse_file_with_stats<P: AsRef<Path>>(
    path: P,
) -> Result<(Graph, ParseStats), ZoteroRdfError> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    parse_reader_with_stats(reader, DEFAULT_BASE_IRI)
}

/// Parses from a Reader and returns detailed statistics
#[instrument(skip(reader), fields(base_iri = %base_iri))]
pub fn parse_reader_with_stats<R: Read>(
    reader: R,
    base_iri: &str,
) -> Result<(Graph, ParseStats), ZoteroRdfError> {
    let mut graph = Graph::default();
    let mut stats = ParseStats::default();

    let parser = RdfXmlParser::new()
        .with_base_iri(base_iri)
        .map_err(|e| ZoteroRdfError::InvalidUri { uri: e.to_string() })?;

    for triple_result in parser.for_reader(reader) {
        match triple_result {
            Ok(triple) => {
                graph.insert(triple.as_ref());
                stats.triples_count += 1;
            }
            Err(e) => {
                stats.error_count += 1;
                warn!("Failed to parse triple: {}", e);

                if stats.error_count >= 100 {
                    return Err(ZoteroRdfError::parse_error(format!(
                        "Too many parse errors ({}), stopping",
                        stats.error_count
                    )));
                }
            }
        }
    }

    info!(
        "RDF parsing complete: {} triples, {} errors",
        stats.triples_count, stats.error_count
    );

    Ok((graph, stats))
}
