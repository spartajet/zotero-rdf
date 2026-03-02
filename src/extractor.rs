use crate::model::{Attachment, Author, ZoteroItem};
use crate::vocab;
use oxrdf::{Graph, Subject, Term};
use std::collections::HashSet;

/// RDF 图提取器，用于将原始 RDF 数据转换为结构化的 ZoteroItem
pub struct Extractor<'a> {
    graph: &'a Graph,
}

impl<'a> Extractor<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    /// 从 Graph 中提取所有 Zotero 条目（不包括 attachment）
    pub fn extract_all(&self) -> Vec<ZoteroItem> {
        let mut items = Vec::new();
        let mut processed_subjects: HashSet<Subject> = HashSet::new();

        for triple in self.graph.iter() {
            if triple.predicate == *vocab::Z_ITEM_TYPE {
                let subject: Subject = triple.subject.into();
                // 确保不重复处理同一个 Subject
                if processed_subjects.contains(&subject) {
                    continue;
                }
                processed_subjects.insert(subject.clone());

                // 获取 item_type 并跳过 attachment 类型
                if let Some(item_type) = self.get_literal(&subject, &vocab::Z_ITEM_TYPE) {
                    if item_type == "attachment" {
                        continue; // 跳过 attachment，它们会作为主条目的字段处理
                    }
                }

                if let Some(item) = self.extract_item(&subject) {
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
        let abstract_note = self.get_literal(subject, &vocab::DCTERMS_ABSTRACT);

        // DOI 提取：优先从 bibo:doi 获取，否则从 URI 中提取
        let doi = self.get_literal(subject, &vocab::BIBO_DOI)
            .or_else(|| extract_doi_from_uri(&uri));

        // 3. 复杂属性：作者 (关键点)
        let authors = self.extract_authors(subject);

        // 4. 提取附件
        let attachments = self.extract_attachments(subject);

        Some(ZoteroItem {
            uri,
            item_type,
            title,
            authors,
            date,
            doi,
            abstract_note,
            attachments,
        })
    }

    /// 提取并排序作者
    fn extract_authors(&self, subject: &Subject) -> Vec<Author> {
        let mut indexed_authors: Vec<(u32, Author)> = Vec::new();

        // A. 查找 bib:authors 的目标 (Zotero 使用 bib:authors 而非 dc:creator)
        if let Some(creator_obj) = self.get_object(subject, &vocab::BIB_AUTHORS) {
            // Zotero 结构: Item -> bib:authors -> rdf:Seq
            // 检查目标是否是一个 Seq 容器
            if self.is_rdf_seq(&creator_obj) {
                // B. 解析 Seq 容器中的有序元素 (rdf:_1, rdf:_2 ...)
                // 需要从 Term 中提取 Subject
                if let Some(seq_subject) = term_to_subject(&creator_obj) {
                    for triple in self.graph.triples_for_subject(&seq_subject) {
                        let pred_str = triple.predicate.as_str();
                        // 检查是否是 rdf:_n 格式
                        if let Some(index) = vocab::parse_rdf_li_index(pred_str) {
                            let person_term: Term = triple.object.into();
                            if let Some(author) = self.extract_person_from_term(&person_term) {
                                indexed_authors.push((index, author));
                            }
                        }
                    }
                }
            } else {
                // 兼容处理：如果只有一个作者，可能没有 Seq 包裹，直接是 Person 节点
                if let Some(author) = self.extract_person_from_term(&creator_obj) {
                    indexed_authors.push((1, author));
                }
            }
        }

        // 按索引排序
        indexed_authors.sort_by_key(|k| k.0);
        indexed_authors.into_iter().map(|(_, a)| a).collect()
    }

    /// 提取附件列表
    fn extract_attachments(&self, subject: &Subject) -> Vec<Attachment> {
        let mut attachments = Vec::new();

        // 查找所有 link:link 关联的附件
        for triple in self.graph.triples_for_subject(subject) {
            if triple.predicate == vocab::LINK_LINK.as_ref() {
                // 获取附件的 URI
                if let oxrdf::TermRef::NamedNode(nn) = &triple.object {
                    // 直接使用 NamedNode 创建 Subject
                    let subject = Subject::from(nn.clone());
                    if let Some(attachment) = self.extract_attachment_from_subject(&subject) {
                        attachments.push(attachment);
                    }
                } else if let oxrdf::TermRef::BlankNode(bn) = &triple.object {
                    // 附件可能是 BlankNode
                    let subject = Subject::from(bn.clone());
                    if let Some(attachment) = self.extract_attachment_from_subject(&subject) {
                        attachments.push(attachment);
                    }
                }
            }
        }

        attachments
    }

    /// 从 Subject 提取附件信息
    fn extract_attachment_from_subject(&self, subject: &Subject) -> Option<Attachment> {
        let uri = subject.to_string();
        let title = self.get_literal(subject, &vocab::DC_TITLE);
        let content_type = self.get_literal(subject, &vocab::LINK_TYPE);

        // 从 dc:identifier 中提取 URL
        let url = self.extract_attachment_url(subject);

        Some(Attachment {
            uri,
            title,
            content_type,
            url,
        })
    }

    /// 从 dc:identifier 中提取附件 URL
    fn extract_attachment_url(&self, subject: &Subject) -> Option<String> {
        // dc:identifier 可能包含 dcterms:URI 节点
        if let Some(identifier_obj) = self.get_object(subject, &vocab::DC_IDENTIFIER) {
            match &identifier_obj {
                Term::BlankNode(bn) => {
                    // 查找 dcterms:URI 节点中的 rdf:value
                    let subject = Subject::from(bn.clone());
                    self.get_literal(&subject, &vocab::RDF_VALUE)
                }
                Term::Literal(lit) => Some(lit.value().to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    fn extract_person_from_term(&self, term: &Term) -> Option<Author> {
        let subject = term_to_subject(term)?;
        self.extract_person(&subject)
    }

    fn extract_person(&self, subject: &Subject) -> Option<Author> {
        let surname = self.get_literal(subject, &vocab::FOAF_SURNAME);
        let given = self.get_literal(subject, &vocab::FOAF_GIVENNAME);

        // 至少需要一个字段才算有效作者
        if surname.is_none() && given.is_none() {
            return None;
        }

        Some(Author {
            surname,
            given_name: given,
            full_name: None,
        })
    }

    /// 获取对象 term
    fn get_object(&self, subject: &Subject, predicate: &oxrdf::NamedNode) -> Option<Term> {
        self.graph
            .object_for_subject_predicate(subject, predicate)
            .map(|t| t.into_owned())
    }

    /// 获取字面量值
    fn get_literal(&self, subject: &Subject, predicate: &oxrdf::NamedNode) -> Option<String> {
        self.get_object(subject, predicate).and_then(|obj| {
            if let Term::Literal(lit) = obj {
                Some(lit.value().to_string())
            } else {
                None
            }
        })
    }

    /// 检查 term 是否为 rdf:Seq 类型
    fn is_rdf_seq(&self, term: &Term) -> bool {
        match term {
            Term::BlankNode(bn) => self
                .graph
                .object_for_subject_predicate(bn, &*vocab::RDF_TYPE)
                .map(|t| t == oxrdf::TermRef::NamedNode(vocab::RDF_SEQ.as_ref()))
                .unwrap_or(false),
            _ => false,
        }
    }
}

/// 从 Term 中提取 Subject（仅支持 BlankNode 和 NamedNode）
fn term_to_subject(term: &Term) -> Option<Subject> {
    match term {
        Term::BlankNode(bn) => Some(Subject::from(bn.clone())),
        Term::NamedNode(nn) => Some(Subject::from(nn.clone())),
        _ => None,
    }
}

/// 从 URI 中提取 DOI
/// 支持格式：
/// - https://doi.org/10.xxx/yyy
/// - http://dx.doi.org/10.xxx/yyy
fn extract_doi_from_uri(uri: &str) -> Option<String> {
    // 移除 RDF 的尖括号
    let uri = uri.trim_start_matches('<').trim_end_matches('>');

    // 尝试从 doi.org URI 中提取
    if let Some(rest) = uri.strip_prefix("https://doi.org/") {
        return Some(rest.to_string());
    }
    if let Some(rest) = uri.strip_prefix("http://dx.doi.org/") {
        return Some(rest.to_string());
    }
    if let Some(rest) = uri.strip_prefix("https://dx.doi.org/") {
        return Some(rest.to_string());
    }

    None
}
