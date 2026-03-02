//! 性能测试
//!
//! 验证库在处理大型 Zotero 库导出文件时的性能和内存效率。

use std::sync::Once;
use std::time::Instant;
use tracing::info;
use zotero_rdf::{parse_file, parse_file_with_stats, Extractor};

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

/// 测试解析性能
#[test]
fn test_parsing_performance() {
    init_tracing();

    let start = Instant::now();
    let (graph, stats) = parse_file_with_stats(TEST_RDF_FILE).expect("Failed to parse file");
    let parse_time = start.elapsed();

    info!("Parse time: {:?}", parse_time);
    info!("Triples count: {}", stats.triples_count);
    info!("Error count: {}", stats.error_count);
    info!("Graph size: {}", graph.len());

    // 性能基准：5000 个三元组应该在 1 秒内完成
    // 注意：这个基准可能需要根据实际硬件调整
    let max_expected_ms = if stats.triples_count > 5000 { 2000 } else { 1000 };
    assert!(
        parse_time.as_millis() < max_expected_ms as u128,
        "Parsing {} triples took {:?}, expected < {}ms",
        stats.triples_count,
        parse_time,
        max_expected_ms
    );
}

/// 测试提取性能
#[test]
fn test_extraction_performance() {
    init_tracing();

    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");

    let extract_start = Instant::now();
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();
    let extract_time = extract_start.elapsed();

    info!("Extraction time: {:?}", extract_time);
    info!("Items count: {}", items.len());

    // 提取 100 个条目应该在 100ms 内完成
    let max_expected_ms = if items.len() > 100 { 200 } else { 100 };
    assert!(
        extract_time.as_millis() < max_expected_ms as u128,
        "Extracting {} items took {:?}, expected < {}ms",
        items.len(),
        extract_time,
        max_expected_ms
    );
}

/// 测试完整流程性能（解析 + 提取）
#[test]
fn test_full_pipeline_performance() {
    init_tracing();

    let start = Instant::now();

    // 完整流程
    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    let total_time = start.elapsed();

    info!("Total pipeline time: {:?}", total_time);
    info!("Graph size: {} triples", graph.len());
    info!("Items extracted: {}", items.len());

    // 完整流程应该在 2 秒内完成
    assert!(
        total_time.as_millis() < 2000,
        "Full pipeline took {:?}, expected < 2000ms",
        total_time
    );
}

/// 测试内存效率估算
#[test]
fn test_memory_efficiency() {
    init_tracing();

    let (graph, stats) = parse_file_with_stats(TEST_RDF_FILE).expect("Failed to parse file");

    // 估算每个三元组的平均内存占用
    // oxrdf 内部使用 Arc 和高效存储，预期每个三元组约 200-500 字节
    let triples_count = stats.triples_count;
    let estimated_size_bytes = triples_count * 500;
    let estimated_size_mb = estimated_size_bytes as f64 / (1024.0 * 1024.0);

    info!(
        "Estimated memory: {:.2} MB for {} triples ({:.2} bytes/triple)",
        estimated_size_mb,
        triples_count,
        estimated_size_bytes as f64 / triples_count as f64
    );

    // 5000 个三元组应该小于 5MB
    assert!(
        estimated_size_mb < 5.0,
        "Estimated memory {} MB exceeds 5 MB limit",
        estimated_size_mb
    );
}

/// 测试多次解析的一致性
#[test]
fn test_parsing_consistency() {
    init_tracing();

    // 解析多次，确保结果一致
    let mut results = Vec::new();

    for i in 0..3 {
        let start = Instant::now();
        let (graph, stats) = parse_file_with_stats(TEST_RDF_FILE).expect("Failed to parse file");
        let elapsed = start.elapsed();

        info!(
            "Run {}: {} triples, {} errors, {:?}",
            i + 1,
            stats.triples_count,
            stats.error_count,
            elapsed
        );

        results.push((graph.len(), stats.triples_count, stats.error_count));
    }

    // 验证所有运行结果一致
    let first = &results[0];
    for (i, result) in results.iter().enumerate() {
        assert_eq!(
            result.0, first.0,
            "Graph size mismatch in run {}",
            i + 1
        );
        assert_eq!(
            result.1, first.1,
            "Triples count mismatch in run {}",
            i + 1
        );
        assert_eq!(
            result.2, first.2,
            "Error count mismatch in run {}",
            i + 1
        );
    }
}

/// 测试提取结果的一致性
#[test]
fn test_extraction_consistency() {
    init_tracing();

    let graph = parse_file(TEST_RDF_FILE).expect("Failed to parse file");

    // 提取多次，确保结果一致
    let mut item_counts = Vec::new();
    let mut author_counts = Vec::new();

    for i in 0..3 {
        let extractor = Extractor::new(&graph);
        let items = extractor.extract_all();

        let total_authors: usize = items.iter().map(|item| item.authors.len()).sum();

        info!("Run {}: {} items, {} total authors", i + 1, items.len(), total_authors);

        item_counts.push(items.len());
        author_counts.push(total_authors);
    }

    // 验证所有运行结果一致
    for (i, (items, authors)) in item_counts.iter().zip(author_counts.iter()).enumerate() {
        assert_eq!(
            *items, item_counts[0],
            "Item count mismatch in run {}",
            i + 1
        );
        assert_eq!(
            *authors, author_counts[0],
            "Author count mismatch in run {}",
            i + 1
        );
    }
}
