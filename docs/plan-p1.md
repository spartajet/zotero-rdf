这是 `zotero-rdf` 项目 **Phase 1: 基础解析 (MVP)** 的详细实现计划。该阶段的目标是完成从“Zotero RDF/XML 文件”到“内存中的 `oxrdf::Graph` 对象”的完整闭环，不涉及复杂的数据提取逻辑。
---
## 🎯 P1 阶段目标
1.  **环境搭建**：完成 Cargo 项目初始化与依赖配置。
2.  **基础设施**：定义统一的错误处理类型和命名空间常量。
3.  **核心功能**：实现 `parse_file` 函数，能够成功读取并解析 Zotero 导出的 RDF 文件。
4.  **质量保证**：通过单元测试验证解析结果的正确性（三元组数量、关键谓词存在性）。
---
## 📅 实施步骤清单
### 步骤 1: 项目初始化与依赖配置
**任务描述**：创建 Rust 库项目，并添加核心解析依赖。
**操作指令**：
```bash
cargo new --lib zotero-rdf
cd zotero-rdf
```
**文件修改 (`Cargo.toml`)**：
```toml
[package]
name = "zotero-rdf"
version = "0.1.0"
edition = "2021"
[dependencies]
# 核心数据结构
oxrdf = "0.3"       # 请使用 crates.io 最新版本
oxrdfxml = "0.2"    # 请使用 crates.io 最新版本
# 工具库
thiserror = "1.0"   # 错误处理
once_cell = "1.18"  # 常量定义
[dev-dependencies]
# 测试依赖
```
---
### 步骤 2: 定义错误类型 (`src/error.rs`)
**任务描述**：建立统一的错误枚举，封装 IO 错误和 RDF 解析错误。
**代码实现**：
```rust
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ZoteroRdfError {
    #[error("Failed to read file or stream: {0}")]
    Io(#[from] std::io::Error),
    #[error("RDF/XML parsing failed: {0}")]
    ParseError(String),
    #[error("Invalid URI encountered: {0}")]
    InvalidUri(String),
}
```
---
### 步骤 3: 定义命名空间常量 (`src/vocab.rs`)
**任务描述**：将 Zotero 相关的 URI 硬编码，为后续解析和查询做准备。
**代码实现**：
```rust
use oxrdf::NamedNode;
use once_cell::sync::Lazy;
// --- 命名空间基础 URI ---
pub const NS_ZOTERO: &str = "http://www.zotero.org/namespaces/export#";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
// --- 关键谓词 (用于后续测试验证) ---
pub const RDF_TYPE: Lazy<NamedNode> = Lazy::new(|| {
    NamedNode::new_unchecked(format!("{}type", NS_RDF))
});
pub const Z_ITEM_TYPE: Lazy<NamedNode> = Lazy::new(|| {
    NamedNode::new_unchecked(format!("{}itemType", NS_ZOTERO))
});
pub const DC_TITLE: Lazy<NamedNode> = Lazy::new(|| {
    NamedNode::new_unchecked(format!("{}title", NS_DC))
});
```
---
### 步骤 4: 实现核心解析逻辑 (`src/parser.rs`)
**任务描述**：封装 `oxrdfxml` 引擎，实现文件路径到 Graph 的转换。
**关键点**：
1. 使用 `std::fs::File` 打开文件。
2. 使用 `std::io::BufReader` 提高读取效率。
3. 处理解析迭代过程中的错误，将其转换为 `ZoteroRdfError`。
**代码实现**：
```rust
use crate::error::ZoteroRdfError;
use oxrdf::{Graph, Triple};
use oxrdfxml::RdfXmlParser;
use std::io::Read;
use std::path::Path;
use std::fs::File;
/// 从文件路径解析 RDF
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Graph, ZoteroRdfError> {
    let file = File::open(path)?;
    // 使用 BufReader 提升性能
    let reader = std::io::BufReader::new(file);
    parse_reader(reader)
}
/// 从任意 Reader 解析 RDF (核心逻辑)
pub fn parse_reader<R: Read>(reader: R) -> Result<Graph, ZoteroRdfError> {
    let mut graph = Graph::default();
    
    // 初始化 Parser
    // 注意：根据 oxrdfxml 版本，API 可能略有不同，通常为 RdfXmlParser::new().parse(reader)
    // 或者 RdfXmlParser::from_reader(reader)
    let parser = RdfXmlParser::new().unwrap().parse(reader);
    for quad_result in parser {
        match quad_result {
            Ok(quad) => {
                // Zotero 导出的通常是单图数据，将 Quad 转换为 Triple
                graph.insert(quad.into());
            }
            Err(e) => {
                // 将底层解析错误转换为我们的错误类型
                return Err(ZoteroRdfError::ParseError(e.to_string()));
            }
        }
    }
    Ok(graph)
}
```
---
### 步骤 5: 模块组织 (`src/lib.rs`)
**任务描述**：组织模块结构并导出公共 API。
**代码实现**：
```rust
mod error;
mod vocab;
mod parser;
// --- 导出公共 API ---
pub use error::ZoteroRdfError;
pub use parser::{parse_file, parse_reader};
pub use oxrdf::Graph; // 重导出 Graph，方便用户使用
// 内部使用的词汇表不一定要导出，除非用户需要
// pub use vocab;
```
---
### 步骤 6: 编写测试用例 (TDD)
**任务描述**：准备测试数据并验证解析功能。
**1. 准备测试数据**：
在项目根目录创建 `tests/fixtures` 目录，并放入一个 Zotero 导出的 `test.rdf` 文件。
*(如果暂时没有文件，可先编写一个生成测试文件的逻辑，或手动创建一个简单的 RDF/XML 片段)*
**示例 `tests/fixtures/test.rdf`**:
```xml
<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:z="http://www.zotero.org/namespaces/export#"
         xmlns:dc="http://purl.org/dc/elements/1.1/">
  <rdf:Description rdf:about="http://zotero.org/users/123/items/ABCDEF">
    <z:itemType>journalArticle</z:itemType>
    <dc:title>A Test Article Title</dc:title>
  </rdf:Description>
</rdf:RDF>
```
**2. 编写测试代码** (`tests/integration_test.rs` 或 `src/lib.rs` 内部)：
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab;
    #[test]
    fn test_parse_zotero_file_success() {
        // 1. 准备路径
        let path = "tests/fixtures/test.rdf"; // 确保文件存在
        
        // 2. 执行解析
        let result = parse_file(path);
        
        // 3. 断言解析成功
        assert!(result.is_ok(), "Failed to parse file: {:?}", result.err());
        let graph = result.unwrap();
        // 4. 验证内容
        // 检查三元组数量 (示例文件包含 item type 和 title，应该至少有 2 个三元组)
        assert!(graph.len() >= 2, "Graph should contain triples");
        // 5. 验证特定谓词是否存在
        // 查询 z:itemType 是否存在
        let has_item_type = graph.iter().any(|t| t.predicate == *vocab::Z_ITEM_TYPE);
        assert!(has_item_type, "Graph should contain z:itemType predicate");
    }
    #[test]
    fn test_parse_invalid_path() {
        let result = parse_file("non/existent/path.rdf");
        assert!(result.is_err());
        match result.unwrap_err() {
            ZoteroRdfError::Io(_) => (), // 预期 IO 错误
            _ => panic!("Expected IO error"),
        }
    }
}
```
---
## ✅ P1 验收标准
在完成上述步骤后，项目应满足以下标准方可进入下一阶段：
1.  **编译通过**：`cargo build` 无错误、无警告。
2.  **测试通过**：`cargo test` 全部通过，能够成功解析标准 Zotero RDF 文件。
3.  **结果验证**：打印 `graph.len()` 显示非零值，且能查询到 `z:itemType` 谓词。
4.  **API 稳定**：`parse_file` 的函数签名确定，后续不再轻易变动。
---
## 🛠️ 开发者备忘录
*   **依赖版本**：`oxrdf` 和 `oxrdfxml` 更新较快，如果编译报错，请检查 `Cargo.toml` 中的版本号是否与文档一致，或查看 `crates.io` 最新版本文档。
*   **性能考量**：P1 阶段主要关注正确性，对于 GB 级别的大文件，流式解析逻辑已在 `parse_reader` 中实现，但内存占用取决于 `Graph` 的大小。
*   **调试技巧**：如果解析出错，可以在 `parser.rs` 中打印错误详情 `e`，RDF 解析错误通常会包含行号和列号。
