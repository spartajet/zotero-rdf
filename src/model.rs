use serde::{Deserialize, Serialize};

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
