use crate::error::ZoteroRdfError;
use oxrdf::Graph;
use oxrdfxml::RdfXmlParser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::{debug, info, instrument, warn};

/// 默认的 base IRI，用于解析 Zotero 导出文件中的相对 IRI
pub const DEFAULT_BASE_IRI: &str = "http://zotero.org/export#";

/// 从文件路径解析 Zotero RDF 文件到内存图
#[instrument(skip(path), fields(path = %path.as_ref().display()))]
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Graph, ZoteroRdfError> {
    info!("开始解析 RDF 文件");
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    parse_reader(reader)
}

/// 从文件路径解析 Zotero RDF 文件，使用指定的 base IRI
#[instrument(skip(path), fields(path = %path.as_ref().display(), base_iri = %base_iri))]
pub fn parse_file_with_base<P: AsRef<Path>>(path: P, base_iri: &str) -> Result<Graph, ZoteroRdfError> {
    info!("开始解析 RDF 文件，base_iri: {}", base_iri);
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    parse_reader_with_base(reader, base_iri)
}

/// 从任意 Reader 解析 RDF (核心逻辑)
pub fn parse_reader<R: Read>(reader: R) -> Result<Graph, ZoteroRdfError> {
    parse_reader_with_base(reader, DEFAULT_BASE_IRI)
}

/// 从任意 Reader 解析 RDF，使用指定的 base IRI
#[instrument(skip(reader), fields(base_iri = %base_iri))]
pub fn parse_reader_with_base<R: Read>(reader: R, base_iri: &str) -> Result<Graph, ZoteroRdfError> {
    let mut graph = Graph::default();

    // 使用 oxrdfxml 解析器，设置 base IRI 以解析相对 IRI
    let parser = RdfXmlParser::new()
        .with_base_iri(base_iri)
        .map_err(|e| ZoteroRdfError::InvalidUri(e.to_string()))?;

    // for_reader 返回 Triple 迭代器
    let mut triple_count = 0;
    let mut error_count = 0;

    for triple_result in parser.for_reader(reader) {
        match triple_result {
            Ok(triple) => {
                graph.insert(&triple);
                triple_count += 1;
            }
            Err(e) => {
                error_count += 1;
                warn!("解析三元组失败: {}", e);
                // 继续解析其他三元组，而不是直接返回错误
                if error_count > 100 {
                    return Err(ZoteroRdfError::ParseError(format!(
                        "解析错误过多 ({}), 停止解析",
                        error_count
                    )));
                }
            }
        }
    }

    info!(
        "RDF 解析完成: {} 个三元组, {} 个错误",
        triple_count, error_count
    );
    debug!("Graph 包含 {} 个三元组", graph.len());

    Ok(graph)
}
