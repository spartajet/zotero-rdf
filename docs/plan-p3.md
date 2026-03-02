这是 `zotero-rdf` 项目 **Phase 3: 健壮性与优化 (Robustness & Optimization)** 的详细实施计划。

P3 阶段的目标是提升库的健壮性、错误诊断能力和性能，使其能够更好地处理真实世界中的各种边界情况。

---

## 🎯 P3 阶段目标

1.  **错误增强**：完善错误类型，提供更详细的错误上下文（文件位置、行列号等）。
2.  **编码处理**：确保正确处理各种字符编码，特别是中文等非 ASCII 字符。
3.  **性能优化**：增加大文件测试，验证流式解析的内存效率。
4.  **API 文档**：完善公开 API 的文档注释。

---

## 📅 实施步骤清单

### 步骤 1: 增强错误类型 (`src/error.rs`)

**任务描述**：扩展 `ZoteroRdfError` 以包含更丰富的错误上下文信息，帮助用户快速定位问题。

**当前错误类型**：
```rust
#[derive(Error, Debug)]
pub enum ZoteroRdfError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    #[error("RDF/XML parsing failed: {0}")]
    ParseError(String),
}
```

**增强后的错误类型**：
```rust
use thiserror::Error;

/// 解析过程中的位置信息
#[derive(Debug, Clone)]
pub struct ErrorLocation {
    /// 字节偏移量
    pub byte_offset: Option<usize>,
    /// 行号 (1-based)
    pub line: Option<usize>,
    /// 列号 (1-based)
    pub column: Option<usize>,
}

impl std::fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(col)) => write!(f, "line {}, column {}", line, col),
            (Some(line), None) => write!(f, "line {}", line),
            (None, Some(col)) => write!(f, "column {}", col),
            (None, None) => write!(f, "unknown position"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ZoteroRdfError {
    /// IO 错误（文件不存在、权限问题等）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 无效的 URI/IRI
    #[error("Invalid URI: {uri}")]
    InvalidUri {
        uri: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// RDF/XML 解析错误
    #[error("RDF/XML parse error at {location}: {message}")]
    ParseError {
        message: String,
        location: ErrorLocation,
    },

    /// 字符编码错误
    #[error("Encoding error at {location}: {message}")]
    EncodingError {
        message: String,
        location: ErrorLocation,
    },

    /// 缺少必需字段
    #[error("Missing required field '{field}' in {context}")]
    MissingField {
        field: String,
        context: String,
    },

    /// 不支持的特性
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

/// 便于创建解析错误的辅助函数
impl ZoteroRdfError {
    pub fn parse_error(message: impl Into<String>) -> Self {
        ZoteroRdfError::ParseError {
            message: message.into(),
            location: ErrorLocation {
                byte_offset: None,
                line: None,
                column: None,
            },
        }
    }

    pub fn parse_error_with_location(message: impl Into<String>, line: usize, column: usize) -> Self {
        ZoteroRdfError::ParseError {
            message: message.into(),
            location: ErrorLocation {
                byte_offset: None,
                line: Some(line),
                column: Some(column),
            },
        }
    }
}
```

---

### 步骤 2: 验证编码处理

**任务描述**：确保库能正确处理各种字符编码，特别是：
- UTF-8 编码的 RDF 文件（标准情况）
- 包含中文、日文等非 ASCII 字符的内容
- XML 声明中指定了其他编码但实际是 UTF-8 的情况

**测试用例**：
```rust
// tests/encoding_test.rs

/// 测试解析包含中文的 RDF 文件
#[test]
fn test_chinese_characters() {
    let graph = parse_file("rdfs/gear-measure-without-attachments.rdf").unwrap();

    // 验证中文标题正确解析
    let title_predicate = NamedNode::new("http://purl.org/dc/elements/1.1/title").unwrap();

    let mut has_chinese = false;
    for triple in graph.iter() {
        if triple.predicate == title_predicate {
            let obj = triple.object.as_str();
            if obj.chars().any(|c| c > '\u{7F}') {
                has_chinese = true;
                // 确保中文字符没有被截断或乱码
                assert!(obj.len() > 3, "Chinese text should have content");
            }
        }
    }
    assert!(has_chinese, "Test file should contain Chinese characters");
}

/// 测试作者姓名（中文）正确解析
#[test]
fn test_chinese_author_names() {
    let graph = parse_file("rdfs/gear-measure-without-attachments.rdf").unwrap();
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // 查找有中文作者的条目
    let has_chinese_author = items.iter().any(|item| {
        item.authors.iter().any(|a| {
            a.surname.as_ref().map(|s| s.chars().any(|c| c > '\u{7F}')).unwrap_or(false)
            || a.given_name.as_ref().map(|g| g.chars().any(|c| c > '\u{7F}')).unwrap_or(false)
        })
    });

    assert!(has_chinese_author, "Should have Chinese authors");
}
```

---

### 步骤 3: 大文件流式解析测试

**任务描述**：验证库在处理大型 Zotero 库导出文件时的内存效率。

**测试方案**：
```rust
// tests/performance_test.rs
use std::time::Instant;

/// 测试大文件解析性能
#[test]
fn test_large_file_parsing() {
    // 使用现有的测试文件进行基本性能测试
    let start = Instant::now();

    let graph = parse_file("rdfs/gear-measure-without-attachments.rdf").unwrap();
    let parse_time = start.elapsed();

    println!("Parse time: {:?}", parse_time);
    println!("Triples count: {}", graph.len());

    // 性能基准：5000 个三元组应该在 1 秒内完成
    assert!(parse_time.as_millis() < 1000, "Parsing should be fast");

    // 提取测试
    let extract_start = Instant::now();
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();
    let extract_time = extract_start.elapsed();

    println!("Extraction time: {:?}", extract_time);
    println!("Items count: {}", items.len());

    // 提取 100 个条目应该在 100ms 内完成
    assert!(extract_time.as_millis() < 100, "Extraction should be fast");
}

/// 内存使用估算测试
#[test]
fn test_memory_efficiency() {
    let graph = parse_file("rdfs/gear-measure-without-attachments.rdf").unwrap();

    // 估算每个三元组的平均内存占用
    // oxrdf 内部使用 Arc 和高效存储，预期每个三元组约 200-500 字节
    let estimated_size = graph.len() * 500; // bytes
    let estimated_size_mb = estimated_size as f64 / (1024.0 * 1024.0);

    println!("Estimated memory: {:.2} MB for {} triples",
        estimated_size_mb, graph.len());

    // 5000 个三元组应该小于 5MB
    assert!(estimated_size_mb < 5.0, "Memory usage should be reasonable");
}
```

---

### 步骤 4: 完善 API 文档

**任务描述**：为所有公开 API 添加文档注释，生成高质量的 rustdoc。

**需要文档化的模块**：

1. **`src/lib.rs`** - 库级别文档（已部分完成）
2. **`src/parser.rs`** - 解析函数文档
3. **`src/extractor.rs`** - 提取器文档
4. **`src/model.rs`** - 数据模型文档
5. **`src/error.rs`** - 错误类型文档

**文档示例** (`src/parser.rs`)：
```rust
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
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Graph, ZoteroRdfError> {
    // ...
}
```

---

### 步骤 5: 增加错误恢复机制

**任务描述**：改进解析器的容错能力，使其在遇到小错误时能够继续解析。

**当前实现已支持**：
- 解析错误时计数，超过阈值才失败

**增强方向**：
```rust
/// 解析选项，控制错误处理行为
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// 最大允许的错误数量，超过则返回错误
    pub max_errors: usize,
    /// 是否在遇到错误时继续解析
    pub continue_on_error: bool,
    /// 是否记录警告信息
    pub collect_warnings: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_errors: 100,
            continue_on_error: true,
            collect_warnings: true,
        }
    }
}

/// 解析结果，包含成功的数据和可能的警告
#[derive(Debug)]
pub struct ParseResult {
    /// 解析得到的 RDF 图
    pub graph: Graph,
    /// 警告信息列表
    pub warnings: Vec<String>,
    /// 统计信息
    pub stats: ParseStats,
}

#[derive(Debug)]
pub struct ParseStats {
    /// 成功解析的三元组数量
    pub triples_count: usize,
    /// 错误数量
    pub error_count: usize,
    /// 解析耗时（毫秒）
    pub duration_ms: u64,
}
```

---

## ✅ P3 验收标准

1.  **错误诊断**：错误信息包含位置上下文，用户能快速定位问题。
2.  **编码正确**：中文、日文等非 ASCII 字符正确处理，无乱码。
3.  **性能达标**：
    - 5000 三元组解析 < 1s
    - 100 条目提取 < 100ms
    - 内存占用 < 5MB / 5000 三元组
4.  **文档完善**：所有公开 API 都有 rustdoc 文档。
5.  **测试覆盖**：新增编码测试、性能测试用例。

---

## 🛠️ 开发者备忘录

*   **错误位置**：`oxrdfxml` 库可能不直接提供行列号，需要查看其 API 或考虑在错误消息中包含上下文片段。
*   **性能测试**：真实的大文件测试可能需要生成或获取更大的 Zotero 导出文件（10000+ 条目）。
*   **内存优化**：如果内存成为瓶颈，可以考虑：
    - 使用 `oxrdfxml` 的流式 API 直接过滤需要的谓词
    - 实现延迟提取（只提取用户请求的条目）
*   **文档生成**：运行 `cargo doc --open` 检查文档质量。
