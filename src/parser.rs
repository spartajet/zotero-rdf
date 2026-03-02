use crate::error::ZoteroRdfError;
use oxrdf::Graph;
use oxrdfxml::RdfXmlParser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// 默认的 base IRI，用于解析 Zotero 导出文件中的相对 IRI
pub const DEFAULT_BASE_IRI: &str = "http://zotero.org/export#";

/// 从文件路径解析 Zotero RDF 文件到内存图
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Graph, ZoteroRdfError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_reader(reader)
}

/// 从文件路径解析 Zotero RDF 文件，使用指定的 base IRI
pub fn parse_file_with_base<P: AsRef<Path>>(path: P, base_iri: &str) -> Result<Graph, ZoteroRdfError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_reader_with_base(reader, base_iri)
}

/// 从任意 Reader 解析 RDF (核心逻辑)
pub fn parse_reader<R: Read>(reader: R) -> Result<Graph, ZoteroRdfError> {
    parse_reader_with_base(reader, DEFAULT_BASE_IRI)
}

/// 从任意 Reader 解析 RDF，使用指定的 base IRI
pub fn parse_reader_with_base<R: Read>(reader: R, base_iri: &str) -> Result<Graph, ZoteroRdfError> {
    let mut graph = Graph::default();

    // 使用 oxrdfxml 解析器，设置 base IRI 以解析相对 IRI
    let parser = RdfXmlParser::new()
        .with_base_iri(base_iri)
        .map_err(|e| ZoteroRdfError::InvalidUri(e.to_string()))?;

    // for_reader 返回 Triple 迭代器
    for triple_result in parser.for_reader(reader) {
        match triple_result {
            Ok(triple) => {
                graph.insert(&triple);
            }
            Err(e) => {
                return Err(ZoteroRdfError::ParseError(e.to_string()));
            }
        }
    }

    Ok(graph)
}
