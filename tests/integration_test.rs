use zotero_rdf::{parse_file, ZoteroRdfError};
use oxrdf::NamedNode;

const TEST_RDF_FILE: &str = "rdfs/gear-measure-without-attachments.rdf";

/// 测试解析 Zotero RDF 文件成功
#[test]
fn test_parse_zotero_file_success() {
    let result = parse_file(TEST_RDF_FILE);

    assert!(result.is_ok(), "Failed to parse file: {:?}", result.err());
    let graph = result.unwrap();

    // 真实的 Zotero 导出文件应该包含大量三元组
    println!("Graph contains {} triples", graph.len());
    assert!(graph.len() > 100, "Graph should contain many triples, got {}", graph.len());

    // 验证 z:itemType 谓词是否存在
    let item_type_predicate = NamedNode::new("http://www.zotero.org/namespaces/export#itemType").unwrap();
    let has_item_type = graph.iter().any(|t| t.predicate == item_type_predicate);
    assert!(has_item_type, "Graph should contain z:itemType predicate");

    // 验证 dc:title 谓词是否存在
    let title_predicate = NamedNode::new("http://purl.org/dc/elements/1.1/title").unwrap();
    let has_title = graph.iter().any(|t| t.predicate == title_predicate);
    assert!(has_title, "Graph should contain dc:title predicate");
}

/// 测试解析不存在的文件
#[test]
fn test_parse_invalid_path() {
    let result = parse_file("non/existent/path.rdf");
    assert!(result.is_err());

    match result.unwrap_err() {
        ZoteroRdfError::Io(_) => (), // 预期 IO 错误
        _ => panic!("Expected IO error"),
    }
}
