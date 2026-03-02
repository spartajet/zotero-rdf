use oxrdf::NamedNode;
use std::sync::Once;
use tracing::{debug, info, trace};
use zotero_rdf::{Extractor, ZoteroRdfError, parse_file};

const TEST_RDF_FILE: &str = "rdfs/gear-measure-without-attachments.rdf";

static TRACING_INIT: Once = Once::new();

/// 初始化 tracing 日志（只执行一次）
fn init_tracing() {
    TRACING_INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("trace")),
            )
            .with_test_writer()
            .init();
    });
}

/// 测试解析 Zotero RDF 文件成功
#[test]
fn test_parse_zotero_file_success() {
    init_tracing();
    let result = parse_file(TEST_RDF_FILE);

    assert!(result.is_ok(), "Failed to parse file: {:?}", result.err());
    let graph = result.unwrap();

    // 真实的 Zotero 导出文件应该包含大量三元组
    info!("Graph contains {} triples", graph.len());
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
    init_tracing();
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
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    assert!(!items.is_empty(), "Should extract at least one item");

    // 打印所有条目
    info!("========== 共提取 {} 个条目 ==========", items.len());
    for (i, item) in items.iter().enumerate() {
        debug!(
            "[{}] uri={} type={} title={}",
            i + 1,
            item.uri,
            item.item_type,
            item.title.as_deref().unwrap_or("(无标题)")
        );

        if !item.authors.is_empty() {
            let authors: Vec<String> = item.authors.iter().map(|a| a.display_name()).collect();
            trace!("    作者: {}", authors.join(", "));
        }
        if let Some(date) = &item.date {
            trace!("    日期: {}", date);
        }
        if let Some(doi) = &item.doi {
            trace!("    DOI: {}", doi);
        }
        if let Some(abstract_note) = &item.abstract_note {
            // 摘要可能很长，只显示前 100 个字符
            let preview = if abstract_note.chars().count() > 100 {
                format!("{}...", abstract_note.chars().take(100).collect::<String>())
            } else {
                abstract_note.clone()
            };
            trace!("    摘要: {}", preview);
        }
        if !item.attachments.is_empty() {
            for attachment in &item.attachments {
                trace!(
                    "    附件: {} ({})",
                    attachment.title.as_deref().unwrap_or("(无标题)"),
                    attachment.content_type.as_deref().unwrap_or("未知类型")
                );
            }
        }
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
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // 找到有作者的条目
    let item_with_authors = items.iter().find(|item| !item.authors.is_empty());
    if let Some(item) = item_with_authors {
        debug!(
            "Item with {} authors: {}",
            item.authors.len(),
            item.title.as_deref().unwrap_or("")
        );
        // 作者顺序应该保持与 Zotero 导出时一致
        for (i, author) in item.authors.iter().enumerate() {
            trace!("  [{}] {}", i + 1, author.display_name());
        }
    }
}

/// 测试 link:link 谓词解析
#[test]
fn test_link_predicate() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");

    // 查找所有包含 "link" 的谓词
    let mut link_predicates = std::collections::HashSet::new();
    for triple in graph.iter() {
        let pred = triple.predicate.as_str();
        if pred.contains("link") {
            link_predicates.insert(pred.to_string());
        }
    }

    debug!("=== 包含 'link' 的谓词 ===");
    for pred in &link_predicates {
        trace!("  {}", pred);
    }

    // 查找 item_33 的所有 link:link 关联
    debug!("=== item_33 的 link:link 关联 ===");
    let item_33_uri = oxrdf::NamedNode::new("http://zotero.org/export#item_33").unwrap();
    let subject = oxrdf::NamedOrBlankNode::from(item_33_uri);
    for triple in graph.triples_for_subject(&subject) {
        let pred = triple.predicate.as_str();
        if pred == "http://purl.org/rss/1.0/modules/link/link" {
            trace!("  Object: {}", triple.object);
        }
    }

    assert!(!link_predicates.is_empty(), "Should have link predicates");
}
