use oxrdf::NamedNode;
use zotero_rdf::{Extractor, ZoteroRdfError, parse_file};

const TEST_RDF_FILE: &str = "rdfs/gear-measure-without-attachments.rdf";

/// 测试解析 Zotero RDF 文件成功
#[test]
fn test_parse_zotero_file_success() {
    let result = parse_file(TEST_RDF_FILE);

    assert!(result.is_ok(), "Failed to parse file: {:?}", result.err());
    let graph = result.unwrap();

    // 真实的 Zotero 导出文件应该包含大量三元组
    println!("Graph contains {} triples", graph.len());
    assert!(
        graph.len() > 100,
        "Graph should contain many triples, got {}",
        graph.len()
    );

    // 验证 z:itemType 谓词是否存在
    let item_type_predicate =
        NamedNode::new("http://www.zotero.org/namespaces/export#itemType").unwrap();
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

/// 测试提取 Zotero 条目
#[test]
fn test_extract_items() {
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    assert!(!items.is_empty(), "Should extract at least one item");

    // 打印所有条目
    println!("\n========== 共提取 {} 个条目 ==========\n", items.len());
    for (i, item) in items.iter().enumerate() {
        println!("[{}] URI: {}", i + 1, item.uri);
        println!("    类型: {}", item.item_type);
        println!("    标题: {}", item.title.as_deref().unwrap_or("(无标题)"));
        if !item.authors.is_empty() {
            println!("    作者:");
            for author in &item.authors {
                println!("      - {}", author.display_name());
            }
        }
        if let Some(date) = &item.date {
            println!("    日期: {}", date);
        }
        if let Some(doi) = &item.doi {
            println!("    DOI: {}", doi);
        }
        if let Some(abstract_note) = &item.abstract_note {
            // 摘要可能很长，只显示前 100 个字符
            let preview = if abstract_note.chars().count() > 100 {
                format!("{}...", abstract_note.chars().take(100).collect::<String>())
            } else {
                abstract_note.clone()
            };
            println!("    摘要: {}", preview);
        }
        println!();
    }

    let first_item = &items[0];
    // 验证基础字段
    assert!(
        !first_item.item_type.is_empty(),
        "item_type should not be empty"
    );
    assert!(first_item.title.is_some(), "title should exist");
}

/// 测试作者顺序
#[test]
fn test_author_order() {
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // 找到有作者的条目
    let item_with_authors = items.iter().find(|item| !item.authors.is_empty());
    if let Some(item) = item_with_authors {
        println!(
            "Item with {} authors: {}",
            item.authors.len(),
            item.title.as_deref().unwrap_or("")
        );
        // 作者顺序应该保持与 Zotero 导出时一致
        for (i, author) in item.authors.iter().enumerate() {
            println!("  [{}] {}", i + 1, author.display_name());
        }
    }
}
