这是 `zotero-rdf` 库的最终版技术设计文档。该文档整合了需求分析、技术选型、核心命名空间定义以及详细的实现代码结构，专门针对“解析 Zotero 导出的 RDF 文件”这一目标进行了垂直优化。
---
# `zotero-rdf` 库技术设计文档 (最终版)
## 1. 项目概况
### 1.1 项目定位
`zotero-rdf` 是一个专注于解析 Zotero 导出 RDF 文件的 Rust 库。它不追求通用 RDF 解析能力，而是针对 Zotero 的数据结构特点提供高效、强类型的解析与提取接口。
### 1.2 核心范围
- **输入**：Zotero 导出的 RDF/XML 文件（`.rdf`）。
- **输出**：Rust 结构体表示的文献元数据（标题、作者、DOI 等）或底层的 `oxrdf::Graph`。
- **不支持**：Turtle、N-Triples 等非 XML 格式；RDF 写入/序列化功能。
### 1.3 技术栈
| 组件 | 选型 | 说明 |
| :--- | :--- | :--- |
| **核心数据模型** | `oxrdf` | 提供 `Graph`, `Triple`, `Literal` 等基础 RDF 结构。 |
| **底层解析引擎** | `oxrdfxml` | 基于 `quick-xml` 的高性能 RDF/XML 解析器，支持流式解析。 |
| **错误处理** | `thiserror` | 派生错误类型，提供清晰的错误信息。 |
| **惰性初始化** | `once_cell` | 用于全局命名空间常量的高效定义。 |
---
## 2. Zotero 命名空间规范
Zotero RDF/XML 文件混合使用了标准本体和私有命名空间。**解析逻辑必须基于 URI 而非 XML 前缀**。以下是解析过程中必须识别的核心命名空间：
| 命名空间 | 常见前缀 | URI | 用途说明 | 关键谓词/类 |
| :--- | :--- | :--- | :--- | :--- |
| **Zotero Export** | `z:` | `http://www.zotero.org/namespaces/export#` | **核心**。存储 Zotero 特有字段和系统属性。 | `z:itemType`, `z:key`, `z:archive` |
| **Dublin Core** | `dc:` | `http://purl.org/dc/elements/1.1/` | 存储标准元数据（标题、日期等）。 | `dc:title`, `dc:identifier`, `dc:date` |
| **FOAF** | `foaf:` | `http://xmlns.com/foaf/0.1/` | 描述作者与机构。作者通常作为 Blank Node 存在。 | `foaf:Person`, `foaf:surname`, `foaf:givenName` |
| **BIBO** | `bibo:` | `http://purl.org/ontology/bibo/` | 描述引文细节。 | `bibo:doi`, `bibo:isbn`, `bibo:pages` |
| **RDF Syntax** | `rdf:` | `http://www.w3.org/1999/02/22-rdf-syntax-ns#` | RDF 标准语法。 | `rdf:Type`, `rdf:resource` |
---
## 3. 项目结构设计
项目采用极其精简的分层架构，去除不必要的抽象层。
```text
zotero-rdf/
├── Cargo.toml
├── src/
│   ├── lib.rs           # 库入口，导出公共 API
│   ├── error.rs         # 定义 ZoteroRdfError
│   ├── vocab.rs         # 命名空间 URI 常量定义
│   ├── parser.rs        # 核心解析逻辑（封装 oxrdfxml）
│   └── extractor.rs     # 高级数据提取（从 Graph 提取到结构体）
└── tests/
    └── fixtures/        # 存放 Zotero 导出的样例 RDF 文件
        └── journal_article.rdf
```
---
## 4. 核心实现详情
### 4.1 依赖配置 (`Cargo.toml`)
```toml
[package]
name = "zotero-rdf"
version = "0.1.0"
edition = "2021"
[dependencies]
oxrdf = "0.3"
oxrdfxml = "0.2"
thiserror = "1.0"
once_cell = "1.18" # 用于常量定义
```
### 4.2 命名空间常量 (`src/vocab.rs`)
将 URI 硬编码为常量，避免代码中出现“魔法字符串”。
```rust
use oxrdf::NamedNode;
use once_cell::sync::Lazy;
// --- 命名空间基础 URI ---
pub const NS_ZOTERO: &str = "http://www.zotero.org/namespaces/export#";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_FOAF: &str = "http://xmlns.com/foaf/0.1/";
pub const NS_BIBO: &str = "http://purl.org/ontology/bibo/";
// --- 预构造的 NamedNode 常量 (用于高效查询) ---
// Zotero Specifics
pub const Z_ITEM_TYPE: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}itemType", NS_ZOTERO)));
pub const Z_KEY: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}key", NS_ZOTERO)));
// Dublin Core
pub const DC_TITLE: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}title", NS_DC)));
pub const DC_CREATOR: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}creator", NS_DC)));
pub const DC_DATE: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}date", NS_DC)));
// FOAF (Author info)
pub const FOAF_SURNAME: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}surname", NS_FOAF)));
pub const FOAF_GIVENNAME: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}givenName", NS_FOAF)));
// BIBO (Citations)
pub const BIBO_DOI: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}doi", NS_BIBO)));
pub const BIBO_PAGES: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}pages", NS_BIBO)));
```
### 4.3 错误定义 (`src/error.rs`)
```rust
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ZoteroRdfError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("RDF/XML parsing failed: {0}")]
    ParseError(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}
```
### 4.4 核心解析器 (`src/parser.rs`)
这是库的核心入口，负责将文件字节流转换为内存中的图结构。
```rust
use crate::error::ZoteroRdfError;
use oxrdf::{Graph, Quad};
use oxrdfxml::RdfXmlParser;
use std::io::Read;
use std::path::Path;
use std::fs::File;
/// 解析 Zotero RDF 文件到内存图
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Graph, ZoteroRdfError> {
    let file = File::open(path)?;
    parse_reader(file)
}
/// 从任意 Read 流解析
pub fn parse_reader<R: Read>(reader: R) -> Result<Graph, ZoteroRdfError> {
    let mut graph = Graph::default();
    
    // 初始化 oxrdfxml 解析器
    // 注意：实际 API 可能因版本略有不同，此处以 oxrdfxml 0.2+ 为例
    let parser = RdfXmlParser::new().unwrap().parse(reader);
    for quad_result in parser {
        match quad_result {
            Ok(quad) => {
                // Zotero 导出的 RDF 通常没有命名图，直接转换为 Triple
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
### 4.5 高级数据提取 (`src/extractor.rs`)
针对 Zotero 的数据模式（特别是作者结构）提供便捷提取方法。
```rust
use crate::vocab;
use crate::error::ZoteroRdfError;
use oxrdf::{Graph, NamedNode, NamedNodeRef, Term};
/// 表示解析出的 Zotero 条目
pub struct ZoteroItem<'a> {
    graph: &'a Graph,
    subject: NamedNodeRef<'a>,
}
impl<'a> ZoteroItem<'a> {
    pub fn new(graph: &'a Graph, subject: NamedNodeRef<'a>) -> Self {
        Self { graph, subject }
    }
    /// 提取单一属性值
    fn get_literal(&self, predicate: &NamedNode) -> Option<String> {
        self.graph
            .object_for_subject_predicate(self.subject, predicate.into())
            .and_then(|obj| {
                if let Term::Literal(lit) = obj { Some(lit.value().to_string()) } else { None }
            })
    }
    /// 提取标题
    pub fn title(&self) -> Option<String> {
        self.get_literal(&vocab::DC_TITLE)
    }
    /// 提取 DOI
    pub fn doi(&self) -> Option<String> {
        self.get_literal(&vocab::BIBO_DOI)
    }
    
    /// 提取条目类型
    pub fn item_type(&self) -> Option<String> {
        self.get_literal(&vocab::Z_ITEM_TYPE)
    }
    /// 提取作者列表
    /// Zotero 结构: Item -> dc:creator -> BlankNode -> foaf:surname/foaf:givenName
    pub fn authors(&self) -> Vec<String> {
        let mut authors = Vec::new();
        
        // 1. 查找所有 dc:creator 关联的节点
        if let Some(creator_nodes) = self.graph.objects_for_subject_predicate(self.subject, vocab::DC_CREATOR.clone().into()) {
            for creator_node in creator_nodes {
                // 确保是 BlankNode (通常作者是匿名的)
                if let Term::BlankNode(bnode) = creator_node {
                    // 2. 从 BlankNode 提取姓和名
                    let surname = self.graph
                        .object_for_subject_predicate(bnode.into(), vocab::FOAF_SURNAME.clone().into())
                        .and_then(|o| if let Term::Literal(l) = o { Some(l.value().to_string()) } else { None });
                    let given = self.graph
                        .object_for_subject_predicate(bnode.into(), vocab::FOAF_GIVENNAME.clone().into())
                        .and_then(|o| if let Term::Literal(l) = o { Some(l.value().to_string()) } else { None });
                    // 3. 组合姓名
                    let full_name = match (given, surname) {
                        (Some(g), Some(s)) => format!("{}, {}", s, g),
                        (Some(g), None) => g,
                        (None, Some(s)) => s,
                        (None, None) => continue,
                    };
                    authors.push(full_name);
                }
            }
        }
        authors
    }
}
```
### 4.6 库入口 (`src/lib.rs`)
```rust
mod error;
mod vocab;
mod parser;
mod extractor;
// 重导出公共 API
pub use error::ZoteroRdfError;
pub use parser::{parse_file, parse_reader};
pub use extractor::ZoteroItem;
pub use oxrdf::Graph; // 允许用户直接操作底层 Graph
```
---
## 5. 使用示例
用户使用该库的代码将非常简洁：
```rust
use zotero_rdf::{parse_file, ZoteroItem, ZoteroRdfError};
fn main() -> Result<(), ZoteroRdfError> {
    // 1. 解析文件
    let graph = parse_file("tests/fixtures/journal_article.rdf")?;
    
    // 2. 查找图中的主条目 (假设我们知道 URI，或者通过遍历查找包含 z:itemType 的节点)
    // 这里假设 URI 已知，实际项目中可遍历 graph.subjects()
    let subject_uri = "http://zotero.org/users/123/items/ABCDEF"; 
    
    if let Some(subject) = oxrdf::NamedNode::new(subject_uri).ok() {
        let item = ZoteroItem::new(&graph, subject.as_ref());
        
        println!("Title: {:?}", item.title());
        println!("Type: {:?}", item.item_type());
        println!("Authors: {:?}", item.authors());
    }
    Ok(())
}
```
---
## 6. 开发路线图
1.  **Phase 1: 基础解析 (MVP)**
    *   完成 `parser.rs`，成功解析 RDF/XML 为 `Graph`。
    *   编写单元测试，验证标准字段（Title, DOI）可从 Graph 原始查询获取。
2.  **Phase 2: 结构化提取**
    *   实现 `vocab.rs` 常量。
    *   实现 `extractor.rs`，重点解决作者这种复杂结构的提取逻辑。
3.  **Phase 3: 健壮性与优化**
    *   处理 Zotero 导出文件中可能存在的编码问题。
    *   增加大文件流式解析测试。
    *   完善 `Error` 类型，包含行列号信息。
该方案完全满足“仅解析 Zotero RDF”的需求，利用成熟的 `oxrdf` 生态，避免了过度设计，同时通过 `ZoteroItem` 结构体提供了友好的开发体验。
