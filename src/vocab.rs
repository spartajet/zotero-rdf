use once_cell::sync::Lazy;
use oxrdf::NamedNode;

// --- 命名空间基础 URI ---
pub const NS_ZOTERO: &str = "http://www.zotero.org/namespaces/export#";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub const NS_FOAF: &str = "http://xmlns.com/foaf/0.1/";
pub const NS_BIBO: &str = "http://purl.org/ontology/bibo/";

// --- 预构造的 NamedNode 常量 (用于高效查询) ---

// RDF
pub const RDF_TYPE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}type", NS_RDF)));

// Zotero Specifics
pub const Z_ITEM_TYPE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}itemType", NS_ZOTERO)));
pub const Z_KEY: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}key", NS_ZOTERO)));

// Dublin Core
pub const DC_TITLE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}title", NS_DC)));
pub const DC_CREATOR: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}creator", NS_DC)));
pub const DC_DATE: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}date", NS_DC)));
pub const DC_IDENTIFIER: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}identifier", NS_DC)));

// FOAF (Author info)
pub const FOAF_PERSON: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}Person", NS_FOAF)));
pub const FOAF_SURNAME: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}surname", NS_FOAF)));
pub const FOAF_GIVENNAME: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}givenName", NS_FOAF)));

// BIBO (Citations)
pub const BIBO_DOI: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}doi", NS_BIBO)));
pub const BIBO_PAGES: Lazy<NamedNode> =
    Lazy::new(|| NamedNode::new_unchecked(format!("{}pages", NS_BIBO)));
