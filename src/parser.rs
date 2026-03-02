use crate::error::{ParseOptions, ParseStats, ZoteroRdfError};
use oxrdf::Graph;
use oxrdfxml::RdfXmlParser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::{debug, info, instrument, warn};

/// 默认的 base IRI，用于解析 Zotero 导出文件中的相对 IRI
///
/// Zotero 导出的 RDF 文件通常使用相对 URI（如 `#item_123`），
/// 此 base IRI 会被用于解析这些相对引用。
pub const DEFAULT_BASE_IRI: &str = "http://zotero.org/export#";

/// 从文件路径解析 Zotero RDF 文件到内存图
///
/// # Arguments
///
/// * `path` - RDF 文件的路径，可以是相对路径或绝对路径
///
/// # Returns
///
/// 成功时返回 `oxrdf::Graph`，失败时返回 `ZoteroRdfError`
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
/// 可能返回以下错误：
/// - `ZoteroRdfError::Io` - 文件不存在或无法读取
/// - `ZoteroRdfError::ParseError` - RDF/XML 格式错误
/// - `ZoteroRdfError::InvalidUri` - 文件中的 URI 格式无效
#[instrument(skip(path), fields(path = %path.as_ref().display()))]
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Graph, ZoteroRdfError> {
    parse_file_with_options(path, ParseOptions::default())
}

/// 从文件路径解析 Zotero RDF 文件，使用指定的 base IRI
///
/// # Arguments
///
/// * `path` - RDF 文件的路径
/// * `base_iri` - 用于解析相对 URI 的 base IRI
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

/// 使用自定义选项解析文件
///
/// # Arguments
///
/// * `path` - RDF 文件的路径
/// * `options` - 解析选项
///
/// # Example
///
/// ```rust,no_run
/// use zotero_rdf::{parse_file_with_options, ParseOptions};
///
/// let options = ParseOptions::lenient(); // 宽松模式
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

/// 使用自定义选项和 base IRI 解析文件
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

/// 从任意 Reader 解析 RDF (核心逻辑)
///
/// 使用默认的 base IRI (`http://zotero.org/export#`)
pub fn parse_reader<R: Read>(reader: R) -> Result<Graph, ZoteroRdfError> {
    parse_reader_with_options(reader, DEFAULT_BASE_IRI, ParseOptions::default())
}

/// 从任意 Reader 解析 RDF，使用指定的 base IRI
pub fn parse_reader_with_base<R: Read>(
    reader: R,
    base_iri: &str,
) -> Result<Graph, ZoteroRdfError> {
    parse_reader_with_options(reader, base_iri, ParseOptions::default())
}

/// 使用自定义选项从 Reader 解析 RDF
#[instrument(skip(reader), fields(base_iri = %base_iri))]
pub fn parse_reader_with_options<R: Read>(
    reader: R,
    base_iri: &str,
    options: ParseOptions,
) -> Result<Graph, ZoteroRdfError> {
    let mut graph = Graph::default();

    // 使用 oxrdfxml 解析器，设置 base IRI 以解析相对 IRI
    let parser = RdfXmlParser::new()
        .with_base_iri(base_iri)
        .map_err(|e| ZoteroRdfError::InvalidUri { uri: e.to_string() })?;

    // for_reader 返回 Triple 迭代器
    let mut stats = ParseStats::default();

    for triple_result in parser.for_reader(reader) {
        match triple_result {
            Ok(triple) => {
                graph.insert(&triple);
                stats.triples_count += 1;
            }
            Err(e) => {
                stats.error_count += 1;
                warn!("解析三元组失败: {}", e);

                if !options.continue_on_error || stats.error_count >= options.max_errors {
                    return Err(ZoteroRdfError::parse_error(format!(
                        "解析错误过多 ({}), 停止解析。最后错误: {}",
                        stats.error_count, e
                    )));
                }
            }
        }
    }

    info!(
        "RDF 解析完成: {} 个三元组, {} 个错误",
        stats.triples_count, stats.error_count
    );
    debug!("Graph 包含 {} 个三元组", graph.len());

    Ok(graph)
}

/// 解析文件并返回详细统计信息
///
/// # Returns
///
/// 返回元组 `(Graph, ParseStats)`，包含解析结果和统计信息
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

/// 从 Reader 解析并返回详细统计信息
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
                graph.insert(&triple);
                stats.triples_count += 1;
            }
            Err(e) => {
                stats.error_count += 1;
                warn!("解析三元组失败: {}", e);

                if stats.error_count >= 100 {
                    return Err(ZoteroRdfError::parse_error(format!(
                        "解析错误过多 ({}), 停止解析",
                        stats.error_count
                    )));
                }
            }
        }
    }

    info!(
        "RDF 解析完成: {} 个三元组, {} 个错误",
        stats.triples_count, stats.error_count
    );

    Ok((graph, stats))
}
