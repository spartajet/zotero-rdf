//! Zotero 数据模型
//!
//! 本模块定义了 Zotero 条目的数据结构，包括：
//! - [`ZoteroItem`]: 代表一个文献条目（期刊文章、书籍等）
//! - [`Author`]: 代表作者信息
//! - [`Attachment`]: 代表附件信息（PDF 等）

use serde::{Deserialize, Serialize};

/// 代表一个 Zotero 条目（期刊文章、书籍等）
///
/// `ZoteroItem` 是从 RDF 图中提取的结构化数据，包含了文献的主要元数据。
///
/// # Fields
///
/// * `uri` - 条目的唯一标识符（RDF 中的 Subject URI）
/// * `item_type` - Zotero 条目类型（如 journalArticle, book, thesis）
/// * `title` - 标题
/// * `authors` - 作者列表，保持原始顺序
/// * `date` - 出版日期
/// * `doi` - DOI 标识符
/// * `abstract_note` - 摘要
/// * `attachments` - 附件列表（PDF 等）
///
/// # Example
///
/// ```rust
/// use zotero_rdf::{ZoteroItem, Author, Attachment};
///
/// let item = ZoteroItem {
///     uri: "http://example.org/item/1".to_string(),
///     item_type: "journalArticle".to_string(),
///     title: Some("A Research Paper".to_string()),
///     authors: vec![Author {
///         surname: Some("Doe".to_string()),
///         given_name: Some("John".to_string()),
///         full_name: None,
///     }],
///     date: Some("2024".to_string()),
///     doi: Some("10.1234/example".to_string()),
///     abstract_note: Some("This is an abstract.".to_string()),
///     attachments: vec![],
/// };
///
/// assert_eq!(item.authors[0].display_name(), "Doe, John");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoteroItem {
    /// 条目 URI (主键)
    ///
    /// 在 Zotero RDF 中，这是条目的唯一标识符，
    /// 通常格式为 `http://zotero.org/export#item_XXX`
    pub uri: String,

    /// Zotero 条目类型
    ///
    /// 常见类型包括：
    /// - `journalArticle` - 期刊文章
    /// - `book` - 书籍
    /// - `thesis` - 学位论文
    /// - `conferencePaper` - 会议论文
    /// - `report` - 报告
    pub item_type: String,

    /// 标题
    pub title: Option<String>,

    /// 作者列表 (保持原有顺序)
    ///
    /// 作者顺序与 Zotero 库中的顺序一致，
    /// 通过解析 RDF 中的 `rdf:Seq` 结构实现。
    pub authors: Vec<Author>,

    /// 出版日期
    ///
    /// 日期格式可能是年份（如 `2024`）或完整日期（如 `2024-01-15`）。
    pub date: Option<String>,

    /// DOI (Digital Object Identifier)
    ///
    /// DOI 可能从 `bibo:doi` 谓词直接提取，
    /// 或从条目 URI（如 `https://doi.org/10.xxx/yyy`）中解析。
    pub doi: Option<String>,

    /// 摘要
    pub abstract_note: Option<String>,

    /// 附件列表 (PDF 等)
    ///
    /// 附件通过 `link:link` 谓词与主条目关联。
    /// 一个条目可能有多个附件（如 PDF、HTML 快照等）。
    pub attachments: Vec<Attachment>,
}

/// 代表作者信息
///
/// 作者信息从 FOAF 本体中提取，包括姓（surname）和名（givenName）。
///
/// # Example
///
/// ```
/// use zotero_rdf::Author;
///
/// let author = Author {
///     surname: Some("Smith".to_string()),
///     given_name: Some("Jane".to_string()),
///     full_name: None,
/// };
///
/// assert_eq!(author.display_name(), "Smith, Jane");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// 名（Given Name）
    ///
    /// 从 `foaf:givenName` 提取。
    pub given_name: Option<String>,

    /// 姓（Surname）
    ///
    /// 从 `foaf:surname` 提取。
    pub surname: Option<String>,

    /// 全名
    ///
    /// 某些情况下 Zotero 可能直接提供完整姓名。
    pub full_name: Option<String>,
}

impl Author {
    /// 生成标准引用格式的姓名
    ///
    /// 格式为 "姓, 名"（如 "Smith, Jane"）。
    /// 如果只有姓或名，则只返回那一个。
    /// 如果都没有，返回 `full_name` 或空字符串。
    ///
    /// # Example
    ///
    /// ```
    /// use zotero_rdf::Author;
    ///
    /// let author = Author {
    ///     surname: Some("李".to_string()),
    ///     given_name: Some("明".to_string()),
    ///     full_name: None,
    /// };
    ///
    /// assert_eq!(author.display_name(), "李, 明");
    /// ```
    pub fn display_name(&self) -> String {
        match (&self.surname, &self.given_name) {
            (Some(s), Some(g)) => format!("{}, {}", s, g),
            (Some(s), None) => s.clone(),
            (None, Some(g)) => g.clone(),
            (None, None) => self.full_name.clone().unwrap_or_default(),
        }
    }
}

/// 代表附件信息（PDF 等）
///
/// 附件通过 `link:link` 谓词与主条目关联。
/// 一个 Zotero 条目可能有多个附件，如：
/// - PDF 全文
/// - HTML 网页快照
/// - 纯文本笔记
///
/// # Example
///
/// ```
/// use zotero_rdf::Attachment;
///
/// let attachment = Attachment {
///     uri: "http://example.org/attachment/1".to_string(),
///     title: Some("paper.pdf".to_string()),
///     content_type: Some("application/pdf".to_string()),
///     url: Some("files/paper.pdf".to_string()),
/// };
///
/// assert_eq!(attachment.content_type, Some("application/pdf".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// 附件 URI
    ///
    /// 在 RDF 中，这是附件节点的唯一标识符。
    pub uri: String,

    /// 附件标题（通常是文件名）
    ///
    /// 例如："Doe et al_2024_Research Paper.pdf"
    pub title: Option<String>,

    /// 附件类型（MIME 类型）
    ///
    /// 常见类型：
    /// - `application/pdf` - PDF 文件
    /// - `text/html` - HTML 网页
    /// - `text/plain` - 纯文本
    pub content_type: Option<String>,

    /// 附件链接（文件路径或 URL）
    ///
    /// 可能是：
    /// - 相对文件路径（如 `files/paper.pdf`）
    /// - 绝对文件路径
    /// - HTTP URL
    pub url: Option<String>,
}
