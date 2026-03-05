//! 编码处理测试
//!
//! 验证库能正确处理各种字符编码，特别是非 ASCII 字符（如带重音的拉丁字符）。

use oxrdf::NamedNode;
use serde_json;
use std::sync::Once;
use tracing::{debug, info};
use zotero_rdf::{Extractor, parse_file};

// const TEST_RDF_FILE: &str = "rdfs/gear-measure-without-attachments.rdf";
//
const TEST_RDF_FILE: &str = r"C:\Users\guo\Downloads\我的文库\我的文库.rdf";

static TRACING_INIT: Once = Once::new();

fn init_tracing() {
    TRACING_INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_test_writer()
            .init();
    });
}

/// 检查字符是否为非 ASCII 字符（带重音等）
fn is_non_ascii_letter(c: char) -> bool {
    // Latin-1 Supplement (U+0080-U+00FF): 包含带重音的字母如 ç, é, ü 等
    // Latin Extended-A (U+0100-U+017F): 更多带重音的字母
    ('\u{00C0}'..='\u{00FF}').contains(&c) || ('\u{0100}'..='\u{017F}').contains(&c)
}

/// 测试解析包含非 ASCII 字符的 RDF 文件
#[test]
fn test_chinese_characters_in_content() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");

    // 验证非 ASCII 字符正确解析
    let title_predicate = NamedNode::new("http://purl.org/dc/elements/1.1/title").unwrap();

    let mut has_non_ascii = false;
    let mut non_ascii_titles: Vec<String> = Vec::new();

    for triple in graph.iter() {
        if triple.predicate == title_predicate {
            let obj_str = format!("{}", triple.object);
            // 检查是否包含非 ASCII 字符
            if obj_str.chars().any(is_non_ascii_letter) {
                has_non_ascii = true;
                // 确保字符没有被截断或乱码
                assert!(obj_str.len() > 3, "Text should have content");
                non_ascii_titles.push(obj_str);
            }
        }
    }

    info!(
        "Found {} titles with non-ASCII characters",
        non_ascii_titles.len()
    );
    for title in non_ascii_titles.iter().take(5) {
        debug!("Sample title with non-ASCII: {}", title);
    }

    assert!(
        has_non_ascii,
        "Test file should contain non-ASCII characters"
    );
}

/// 测试作者姓名（包含非 ASCII 字符）正确解析
#[test]
fn test_chinese_author_names() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // 查找有非 ASCII 作者名的条目
    let items_with_non_ascii: Vec<_> = items
        .iter()
        .filter(|item| {
            item.authors.iter().any(|a| {
                let has_non_ascii_surname = a
                    .surname
                    .as_ref()
                    .map(|s| s.chars().any(is_non_ascii_letter))
                    .unwrap_or(false);
                let has_non_ascii_given = a
                    .given_name
                    .as_ref()
                    .map(|g| g.chars().any(is_non_ascii_letter))
                    .unwrap_or(false);
                has_non_ascii_surname || has_non_ascii_given
            })
        })
        .collect();

    info!(
        "Found {} items with non-ASCII author names",
        items_with_non_ascii.len()
    );

    // 打印一些示例
    for item in items_with_non_ascii.iter().take(3) {
        debug!(
            "Item with non-ASCII authors: {}",
            item.title.as_deref().unwrap_or("(no title)")
        );
        for author in &item.authors {
            debug!("  - {}", author.display_name());
        }
    }

    assert!(
        !items_with_non_ascii.is_empty(),
        "Should have items with non-ASCII author names"
    );
}

/// 测试摘要中的非 ASCII 字符
#[test]
fn test_chinese_abstract() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // 查找有摘要的条目
    let items_with_abstract: Vec<_> = items
        .iter()
        .filter(|item| item.abstract_note.is_some())
        .collect();

    info!("Found {} items with abstracts", items_with_abstract.len());

    // 验证摘要是完整的（没有截断）
    for item in items_with_abstract.iter().take(3) {
        if let Some(abstract_text) = &item.abstract_note {
            // 摘要应该有合理的长度
            assert!(
                abstract_text.len() > 10,
                "Abstract should have reasonable length"
            );
            debug!(
                "Abstract sample ({} chars): {}...",
                abstract_text.chars().count(),
                abstract_text.chars().take(50).collect::<String>()
            );
        }
    }

    assert!(
        !items_with_abstract.is_empty(),
        "Should have items with abstracts"
    );
}

/// 测试混合语言内容
#[test]
fn test_mixed_language_content() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // 统计不同语言的条目
    let mut non_ascii_count = 0;
    let mut english_count = 0;

    for item in &items {
        if let Some(title) = &item.title {
            let has_non_ascii = title.chars().any(is_non_ascii_letter);
            let has_english = title.chars().any(|c| c.is_ascii_alphabetic());

            if has_non_ascii {
                non_ascii_count += 1;
            } else if has_english {
                english_count += 1;
            }
        }
    }

    info!(
        "Language distribution: Non-ASCII={}, English={}",
        non_ascii_count, english_count
    );

    // 测试文件应该包含英文内容
    assert!(english_count > 0, "Should have English items");
}

/// 测试 JSON 序列化输出
#[test]
fn test_json_output() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    info!("Total items extracted: {}", items.len());

    // 将所有条目序列化为 JSON 并打印
    for (i, item) in items.iter().enumerate() {
        let json = serde_json::to_string_pretty(item).expect("Failed to serialize to JSON");
        info!("Item {}:\n{}", i + 1, json);
    }

    // 确保至少有一些条目被解析
    assert!(!items.is_empty(), "Should have extracted some items");
}
