这是 `zotero-rdf` 项目 **Phase 2: 结构化提取 (Structured Extraction)** 的详细实施计划。
P2 阶段的目标是将 P1 阶段生成的“原始 RDF 图”转换为“具有强类型的 Rust 结构体”，重点解决 Zotero 特有的数据结构映射问题，特别是**作者信息的有序提取**。
---
## 🎯 P2 阶段目标
1.  **模型定义**：定义 `ZoteroItem` 和 `Author` 等 Rust 结构体，承载解析后的业务数据。
2.  **提取逻辑**：实现从 `oxrdf::Graph` 到 `ZoteroItem` 的映射逻辑。
3.  **有序性处理**：解决 RDF 本身无序特性与 Zotero 作者排序需求之间的矛盾（解析 `rdf:Seq` 容器结构）。
4.  **API 完善**：提供 `iter_items()` 或 `extract_all()` 等便捷接口。
---
## 📅 实施步骤清单
### 步骤 1: 定义业务数据模型 (`src/model.rs`)
**任务描述**：创建与 Zotero 字段一一对应的 Rust 结构体。这是库面向用户的最终输出。
**代码实现**：
```rust
use serde::{Serialize, Deserialize}; // 可选：方便用户序列化输出
/// 代表一个 Zotero 条目（期刊文章、书籍等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoteroItem {
    /// 条目 URI (主键)
    pub uri: String,
    /// Zotero 条目类型
    pub item_type: String,
    /// 标题
    pub title: Option<String>,
    /// 作者列表 (保持原有顺序)
    pub authors: Vec<Author>,
    /// 出版日期
    pub date: Option<String>,
    /// DOI
    pub doi: Option<String>,
    /// 摘要
    pub abstract_note: Option<String>,
    // 可根据需要扩展：volume, issue, pages, isbn 等
}
/// 代表作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub full_name: Option<String>,
}
impl Author {
    /// 生成标准引用格式
    pub fn display_name(&self) -> String {
        match (&self.surname, &self.given_name) {
            (Some(s), Some(g)) => format!("{}, {}", s, g),
            (Some(s), None) => s.clone(),
            (None, Some(g)) => g.clone(),
            (None, None) => self.full_name.clone().unwrap_or_default(),
        }
    }
}
```
*(注意：需在 `Cargo.toml` 中添加 `serde` 依赖，如果不需要 JSON 输出功能可省略)*
---
### 步骤 2: 扩展命名空间常量 (`src/vocab.rs`)
**任务描述**：补充 P2 阶段提取所需的谓词 URI，特别是 RDF 容器相关的 URI。
**新增内容**：
```rust
// 在 src/vocab.rs 中补充
pub const NS_RDF: &str = "http://www.z3.org/1999/02/22-rdf-syntax-ns#";
// --- 新增谓词 ---
pub const DC_CREATOR: Lazy<NamedNode> = ...; // dc:creator
pub const DC_DATE: Lazy<NamedNode> = ...;    // dc:date
pub const DC_ABSTRACT: Lazy<NamedNode> = ...; // dc:abstract
pub const BIBO_DOI: Lazy<NamedNode> = ...;    // bibo:doi
// RDF 容器属性 (用于解析有序作者列表)
// Zotero 导出通常使用 rdf:Seq 和 rdf:_1, rdf:_2 ... 来表示顺序
pub const RDF_SEQ: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}Seq", NS_RDF)));
pub const RDF_LI: Lazy<NamedNode> = Lazy::new(|| NamedNode::new_unchecked(format!("{}li", NS_RDF)));
```
---
### 步骤 3: 实现核心提取器 (`src/extractor.rs`)
这是 P2 最复杂的部分。Zotero 导出的作者结构通常遵循 `Item -> dc:creator -> rdf:Seq -> rdf:li -> Person` 的路径。
**关键逻辑**：
1. **发现条目**：遍历 Graph，寻找含有 `z:itemType` 属性的节点。
2. **提取简单字段**：直接查询谓词对象。
3. **提取有序作者**：解析 `rdf:Seq` 结构。
**代码骨架实现**：
```rust
use crate::{vocab, ZoteroItem, Author};
use oxrdf::{Graph, NamedNode, Term, Subject};
pub struct Extractor<'a> {
    graph: &'a Graph,
}
impl<'a> Extractor<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }
    /// 从 Graph 中提取所有 Zotero 条目
    pub fn extract_all(&self) -> Vec<ZoteroItem> {
        let mut items = Vec::new();
        
        // 1. 遍历所有主语，寻找含有 z:itemType 的节点
        // 注意：oxrdf 的 iter() 返回的是 Triple
        let mut processed_subjects = std::collections::HashSet::new();
        
        for triple in self.graph.iter() {
            if triple.predicate == *vocab::Z_ITEM_TYPE {
                // 确保不重复处理同一个 Subject
                if processed_subjects.contains(triple.subject.as_ref()) {
                    continue;
                }
                processed_subjects.insert(triple.subject.as_ref());
                
                if let Some(item) = self.extract_item(&triple.subject) {
                    items.push(item);
                }
            }
        }
        items
    }
    fn extract_item(&self, subject: &Subject) -> Option<ZoteroItem> {
        // 1. 基础信息
        let uri = subject.to_string();
        let item_type = self.get_literal(subject, &vocab::Z_ITEM_TYPE)?;
        
        // 2. 简单属性提取
        let title = self.get_literal(subject, &vocab::DC_TITLE);
        let date = self.get_literal(subject, &vocab::DC_DATE);
        let doi = self.get_literal(subject, &vocab::BIBO_DOI);
        
        // 3. 复杂属性：作者 (关键点)
        let authors = self.extract_authors(subject);
        Some(ZoteroItem {
            uri,
            item_type,
            title,
            authors,
            date,
            doi,
            abstract_note: self.get_literal(subject, &vocab::DC_ABSTRACT),
        })
    }
    /// 提取并排序作者
    fn extract_authors(&self, subject: &Subject) -> Vec<Author> {
        let mut authors = Vec::new();
        
        // A. 查找 dc:creator 的目标
        if let Some(creator_obj) = self.graph.object_for_subject_predicate(subject, vocab::DC_CREATOR.clone().into()) {
            // Zotero 结构: Item -> dc:creator -> rdf:Seq
            // 检查目标是否是一个 Seq 容器
            if self.is_rdf_seq(&creator_obj) {
                // B. 解析 Seq 容器中的有序元素 (rdf:_1, rdf:_2 ...)
                // 这需要遍历该 Seq 节点的所有谓词，匹配 rdf:_\d+
                for triple in self.graph.triples_for_subject(&creator_obj) {
                    if triple.predicate.as_str().starts_with(vocab::NS_RDF) {
                        // 检查是否是 rdf:_n 格式
                        // 简化处理：提取数字后排序，或者直接遍历（依赖底层实现顺序，不安全）
                        // 推荐做法：提取 index，存入 Vec<(i32, Term)>，排序后转换
                        // 此处略去具体解析 rdf:_n 的正则逻辑，直接获取对象
                        let person_node = triple.object;
                        if let Term::BlankNode(bnode) = person_node {
                            if let Some(author) = self.extract_person(&bnode.into()) {
                                authors.push(author);
                            }
                        }
                    }
                }
            } else if let Term::BlankNode(bnode) = creator_obj {
                // 兼容处理：如果只有一个作者，可能没有 Seq 包裹，直接是 Person 节点
                if let Some(author) = self.extract_person(&bnode.into()) {
                    authors.push(author);
                }
            }
        }
        
        // 注意：简单的遍历会丢失顺序。
        // 完善的做法：收集 (order_index, Author) 对，然后排序。
        authors
    }
    fn extract_person(&self, subject: &Subject) -> Option<Author> {
        let surname = self.get_literal(subject, &vocab::FOAF_SURNAME);
        let given = self.get_literal(subject, &vocab::FOAF_GIVENNAME);
        
        Some(Author {
            surname,
            given_name: given,
            full_name: None,
        })
    }
    // 辅助函数：获取字面量
    fn get_literal(&self, subject: &Subject, predicate: &NamedNode) -> Option<String> {
        self.graph
            .object_for_subject_predicate(subject, predicate.clone().into())
            .and_then(|obj| {
                if let Term::Literal(lit) = obj { Some(lit.value().to_string()) } else { None }
            })
    }
    
    fn is_rdf_seq(&self, term: &Term) -> bool {
        // 检查 term 是否具有 rdf:type rdf:Seq
        // 或者在 Zotero RDF 中通常可以通过是否具有 rdf:_1 属性来快速判断
        // 此处简化
        if let Term::BlankNode(bn) = term {
             self.graph.object_for_subject_predicate(bn.into(), vocab::RDF_TYPE.clone().into())
                .map(|t| t == *vocab::RDF_SEQ).unwrap_or(false)
        } else {
            false
        }
    }
}
```
---
### 步骤 4: 解决 RDF Seq 的有序性问题 (技术难点)
RDF 本身是无序图，Zotero 使用 `rdf:Seq` 及 `rdf:_1`, `rdf:_2` 属性来编码顺序。
**具体实现思路**：
1.  在 `extract_authors` 中，不要直接 `push` 到结果列表。
2.  遍历 `rdf:Seq` 节点的属性时，使用正则表达式或字符串解析从谓词 URI 中提取数字索引。
    *   例如：谓词 `http://www.w3.org/1999/02/22-rdf-syntax-ns#_1` 提取出 `1`。
3.  将作者数据存入 `BinaryHeap` 或 `Vec<(u32, Author)>`。
4.  排序后输出。
**代码片段**：
```rust
// 在 extract_authors 函数内部改进
let mut indexed_authors = Vec::new();
// ... 遍历 Seq 谓词 ...
let pred_str = triple.predicate.as_str();
if let Some(idx_str) = pred_str.strip_prefix(format!("{}_", vocab::NS_RDF).as_str()) {
    if let Ok(index) = idx_str.parse::<u32>() {
        if let Some(author) = self.extract_person(...) {
            indexed_authors.push((index, author));
        }
    }
}
// 按索引排序
indexed_authors.sort_by_key(|k| k.0);
authors = indexed_authors.into_iter().map(|(_, a)| a).collect();
```
---
### 步骤 5: 集成测试与 API 导出
**1. 更新 `src/lib.rs`**：
```rust
mod model;
mod extractor;
// ... 其他模块 ...
pub use model::{ZoteroItem, Author};
pub use extractor::Extractor;
```
**2. 编写测试用例 (`tests/extraction_test.rs`)**：
验证字段映射的正确性，特别是作者顺序。
```rust
#[test]
fn test_extraction_logic() {
    // 1. 解析 (复用 P1 功能)
    let graph = parse_file("tests/fixtures/journal_article.rdf").unwrap();
    
    // 2. 提取
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();
    
    assert!(!items.is_empty(), "Should extract at least one item");
    
    let first_item = &items[0];
    assert_eq!(first_item.item_type, "journalArticle");
    assert!(first_item.title.is_some());
    
    // 验证作者顺序
    assert!(first_item.authors.len() > 0, "Should have authors");
    // 假设 fixture 中第一个作者是 "Doe, John"
    assert_eq!(first_item.authors[0].surname, Some("Doe".to_string()));
}
```
---
## ✅ P2 验收标准
1.  **API 可用**：用户可以通过 `Extractor::new(&graph).extract_all()` 获得结构化的 `Vec<ZoteroItem>`。
2.  **字段完整**：核心字段均已正确映射。
3.  **顺序正确**：作者列表的顺序与 Zotero 导出时的顺序一致。
4.  **类型安全**：所有返回值均为强类型，不再需要用户手动处理 RDF 节点类型转换。
---
## 🛠️ 开发者备忘录
*   **性能考量**：`extract_all` 会遍历整个图，对于大型 Zotero 库（数千条目），建议仅在需要时调用。
*   **容错性**：如果某个字段（如 DOI）缺失，应返回 `None` 而不是报错中断整个提取过程。
*   **_blank nodes**：Zotero 的作者节点是匿名的，生命周期仅存在于该 Graph 中。提取阶段必须将这些匿名节点的内容“拷贝”到 `Author` 结构体中，不能持有引用，否则生命周期将绑定到 Graph，导致 API 使用困难。
