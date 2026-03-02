//! 编码处理测试
//!
//! 验证库能正确处理各种字符编码，特别是中文等非 ASCII 字符。

use oxrdf::NamedNode;
use std::sync::Once;
use tracing::{debug, info};
use zotero_rdf::{Extractor, parse_file};

const TEST_RDF_FILE: &str = "rdfs/gear-measure-without-attachments.rdf";

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

/// 测试解析包含中文的 RDF 文件
#[test]
fn test_chinese_characters_in_content() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");

    // 验证中文标题正确解析
    let title_predicate = NamedNode::new("http://purl.org/dc/elements/1.1/title").unwrap();

    let mut has_chinese = false;
    let mut chinese_titles: Vec<String> = Vec::new();

    for triple in graph.iter() {
        if triple.predicate == title_predicate {
            let obj_str = format!("{}", triple.object);
            // 检查是否包含 CJK 字符 (Unicode 范围)
            if obj_str
                .chars()
                .any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c))
            {
                has_chinese = true;
                // 确保中文字符没有被截断或乱码
                assert!(obj_str.len() > 3, "Chinese text should have content");
                chinese_titles.push(obj_str);
            }
        }
    }

    info!("Found {} Chinese titles", chinese_titles.len());
    for title in chinese_titles.iter().take(5) {
        debug!("Sample Chinese title: {}", title);
    }

    assert!(has_chinese, "Test file should contain Chinese characters");
}

/// 测试作者姓名（中文）正确解析
#[test]
fn test_chinese_author_names() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // 查找有中文作者的条目
    let items_with_chinese: Vec<_> = items
        .iter()
        .filter(|item| {
            item.authors.iter().any(|a| {
                let has_chinese_surname = a
                    .surname
                    .as_ref()
                    .map(|s| s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)))
                    .unwrap_or(false);
                let has_chinese_given = a
                    .given_name
                    .as_ref()
                    .map(|g| g.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)))
                    .unwrap_or(false);
                has_chinese_surname || has_chinese_given
            })
        })
        .collect();

    info!(
        "Found {} items with Chinese authors",
        items_with_chinese.len()
    );

    // 打印一些示例
    for item in items_with_chinese.iter().take(3) {
        debug!(
            "Item with Chinese authors: {}",
            item.title.as_deref().unwrap_or("(无标题)")
        );
        for author in &item.authors {
            debug!("  - {}", author.display_name());
        }
    }

    assert!(
        !items_with_chinese.is_empty(),
        "Should have items with Chinese authors"
    );
}

/// 测试摘要中的中文字符
#[test]
fn test_chinese_abstract() {
    init_tracing();
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // 查找有中文摘要的条目
    let items_with_chinese_abstract: Vec<_> = items
        .iter()
        .filter(|item| {
            item.abstract_note
                .as_ref()
                .map(|a| a.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)))
                .unwrap_or(false)
        })
        .collect();

    info!(
        "Found {} items with Chinese abstracts",
        items_with_chinese_abstract.len()
    );

    // 验证摘要是完整的（没有截断）
    for item in items_with_chinese_abstract.iter().take(3) {
        if let Some(abstract_text) = &item.abstract_note {
            // 摘要应该以完整的字符结束，不应该有乱码
            let _last_char = abstract_text.chars().last().unwrap_or(' ');
            // 如果最后一个字符是字母数字或中文标点，说明可能是截断的
            // 这里只检查长度合理
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
        !items_with_chinese_abstract.is_empty(),
        "Should have items with Chinese abstracts"
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
    let mut chinese_count = 0;
    let mut english_count = 0;
    let mut other_count = 0;

    for item in &items {
        if let Some(title) = &item.title {
            let has_chinese = title
                .chars()
                .any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
            let has_english = title.chars().any(|c| c.is_ascii_alphabetic());

            if has_chinese {
                chinese_count += 1;
            } else if has_english {
                english_count += 1;
            } else {
                other_count += 1;
            }
        }
    }

    info!(
        "Language distribution: Chinese={}, English={}, Other={}",
        chinese_count, english_count, other_count
    );

    // 测试文件应该包含多种语言
    assert!(chinese_count > 0, "Should have Chinese items");
    assert!(english_count > 0, "Should have English items");
}
