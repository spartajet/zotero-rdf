use once_cell::sync::Lazy;
use oxrdf::NamedNode;

// --- 命名空间基础 URI ---
pub const NS_ZOTERO: &str = "http://www.zotero.org/namespaces/export#";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_DCTERMS: &str = "http://purl.org/dc/terms/";
pub const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub const NS_FOAF: &str = "http://xmlns.com/foaf/0.1/";
pub const NS_BIBO: &str = "http://purl.org/ontology/bibo/";
pub const NS_BIB: &str = "http://purl.org/net/biblio#";
pub const NS_LINK: &str = "http://purl.org/rss/1.0/modules/link/";

// --- 预构造的 NamedNode 静态变量 (用于高效查询) ---
// 使用 static 而非 const，因为 Lazy<T> 具有内部可变性

// RDF
pub static RDF_TYPE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}type", NS_RDF)));
pub static RDF_SEQ: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}Seq", NS_RDF)));
pub static RDF_VALUE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}value", NS_RDF)));

// Zotero Specifics
pub static Z_ITEM_TYPE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}itemType", NS_ZOTERO)));

// Dublin Core
pub static DC_TITLE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}title", NS_DC)));
pub static DC_DATE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}date", NS_DC)));
pub static DC_IDENTIFIER: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}identifier", NS_DC)));

// Dublin Core Terms
pub static DCTERMS_ABSTRACT: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}abstract", NS_DCTERMS)));

// BIB (Biblio)
pub static BIB_AUTHORS: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}authors", NS_BIB)));

// FOAF (Author info)
pub static FOAF_SURNAME: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}surname", NS_FOAF)));
pub static FOAF_GIVENNAME: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}givenName", NS_FOAF)));

// BIBO (Citations)
pub static BIBO_DOI: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}doi", NS_BIBO)));

// Link (Attachments)
pub static LINK_LINK: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}link", NS_LINK)));
pub static LINK_TYPE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}type", NS_LINK)));

/// 检查谓词是否为 RDF 序数属性 (rdf:_1, rdf:_2, ...)
/// 如果是，返回序号；否则返回 None
pub fn parse_rdf_li_index(predicate: &str) -> Option<u32> {
    let prefix = format!("{}_", NS_RDF);
    predicate
        .strip_prefix(&prefix)
        .and_then(|idx_str| idx_str.parse::<u32>().ok())
}
