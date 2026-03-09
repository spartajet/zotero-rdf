use crate::model::{Attachment, Author, Journal, ZoteroItem};
use crate::vocab;
use oxrdf::{Graph, NamedOrBlankNode, Term};
use std::collections::HashSet;
use tracing::{debug, info, instrument, trace, warn};

/// RDF graph extractor for converting raw RDF data into structured `ZoteroItem` instances
///
/// `Extractor` retrieves Zotero items from a parsed RDF graph, including:
/// - Item types (journalArticle, book, thesis, etc.)
/// - Titles, dates, DOIs, abstracts
/// - Author lists (in original order)
/// - Attachments (PDFs, etc.)
/// - Tags (from dc:subject predicates)
///
/// # Example
///
/// ```rust,no_run
/// use zotero_rdf::{parse_file, Extractor};
///
/// let graph = parse_file("my_library.rdf")?;
/// let extractor = Extractor::new(&graph);
/// let items = extractor.extract_all();
///
/// for item in items {
///     println!("Title: {:?}", item.title);
///     println!("Authors: {}", item.authors.len());
/// }
/// # Ok::<(), zotero_rdf::ZoteroRdfError>(())
/// ```
pub struct Extractor<'a> {
    graph: &'a Graph,
}

impl<'a> Extractor<'a> {
    /// Creates a new extractor
    ///
    /// # Arguments
    ///
    /// * `graph` - Reference to the parsed RDF graph
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    /// Extracts all Zotero items from the graph (excluding attachments)
    ///
    /// Iterates through all nodes in the graph that contain the `z:itemType` predicate
    /// and converts them to `ZoteroItem`. Attachment items (`z:itemType="attachment"`)
    /// are skipped, as they are handled as fields of parent items.
    ///
    /// # Returns
    ///
    /// Returns `Vec<ZoteroItem>` containing all extracted items.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use zotero_rdf::{parse_file, Extractor};
    ///
    /// let graph = parse_file("my_library.rdf")?;
    /// let extractor = Extractor::new(&graph);
    /// let items = extractor.extract_all();
    ///
    /// println!("Found {} items", items.len());
    /// # Ok::<(), zotero_rdf::ZoteroRdfError>(())
    /// ```
    #[instrument(skip(self), fields(graph_len = %self.graph.len()))]
    pub fn extract_all(&self) -> Vec<ZoteroItem> {
        info!("Starting Zotero item extraction");
        let mut items = Vec::new();
        let mut processed_subjects: HashSet<NamedOrBlankNode> = HashSet::new();
        let mut attachment_count = 0;

        for triple in self.graph.iter() {
            if triple.predicate == *vocab::Z_ITEM_TYPE {
                let subject: NamedOrBlankNode = triple.subject.into();
                // Ensure we don't process the same NamedOrBlankNode twice
                if processed_subjects.contains(&subject) {
                    continue;
                }
                processed_subjects.insert(subject.clone());

                // Get item_type and skip attachment types
                if let Some(item_type) = self.get_literal(&subject, &vocab::Z_ITEM_TYPE)
                    && item_type == "attachment"
                {
                    attachment_count += 1;
                    trace!("Skipping attachment item: {}", subject);
                    continue; // Attachments are handled as fields of parent items
                }

                if let Some(item) = self.extract_item(&subject) {
                    items.push(item);
                }
            }
        }

        info!(
            "Extraction complete: {} items, skipped {} attachments",
            items.len(),
            attachment_count
        );
        items
    }

    #[instrument(skip(self), fields(uri = %subject))]
    fn extract_item(&self, subject: &NamedOrBlankNode) -> Option<ZoteroItem> {
        // 1. Basic information
        let uri = subject.to_string();
        let item_type = self.get_literal(subject, &vocab::Z_ITEM_TYPE)?;

        debug!("Extracting item: type={}, uri={}", item_type, uri);

        // 2. Simple property extraction
        let title = self.get_literal(subject, &vocab::DC_TITLE);
        let date = self.get_literal(subject, &vocab::DC_DATE);
        let abstract_note = self.get_literal(subject, &vocab::DCTERMS_ABSTRACT);

        // DOI extraction: prioritize bibo:doi, otherwise extract from URI
        let doi = self
            .get_literal(subject, &vocab::BIBO_DOI)
            .or_else(|| extract_doi_from_uri(&uri));

        // 3. Complex properties: authors (key point)
        let authors = self.extract_authors(subject);

        // 4. Extract attachments
        let attachments = self.extract_attachments(subject);

        if !attachments.is_empty() {
            debug!("Item has {} attachments", attachments.len());
        }

        // 5. Extract tags
        let tags = self.extract_tags(subject);

        // 6. Extract journal information
        let journal = self.extract_journal(subject);

        Some(ZoteroItem {
            uri,
            item_type,
            title,
            authors,
            date,
            doi,
            abstract_note,
            attachments,
            tags,
            journal,
        })
    }

    /// Extracts and sorts authors
    #[instrument(skip(self), fields(uri = %subject))]
    fn extract_authors(&self, subject: &NamedOrBlankNode) -> Vec<Author> {
        let mut indexed_authors: Vec<(u32, Author)> = Vec::new();

        // A. Find the target of bib:authors (Zotero uses bib:authors, not dc:creator)
        if let Some(creator_obj) = self.get_object(subject, &vocab::BIB_AUTHORS) {
            // Zotero structure: Item -> bib:authors -> rdf:Seq
            // Check if the target is a Seq container
            if self.is_rdf_seq(&creator_obj) {
                trace!("Author list uses rdf:Seq structure");
                // B. Parse ordered elements in Seq container (rdf:_1, rdf:_2, ...)
                // Need to extract NamedOrBlankNode from Term
                if let Some(seq_subject) = term_to_subject(&creator_obj) {
                    for triple in self.graph.triples_for_subject(&seq_subject) {
                        let pred_str = triple.predicate.as_str();
                        // Check if it's rdf:_n format
                        if let Some(index) = vocab::parse_rdf_li_index(pred_str) {
                            let person_term: Term = triple.object.into();
                            if let Some(author) = self.extract_person_from_term(&person_term) {
                                trace!("Extracted author [{}]: {}", index, author.display_name());
                                indexed_authors.push((index, author));
                            }
                        }
                    }
                }
            } else {
                // Compatibility: if there's only one author, it might not be wrapped in Seq
                trace!("Author list uses simple structure (no rdf:Seq)");
                if let Some(author) = self.extract_person_from_term(&creator_obj) {
                    indexed_authors.push((1, author));
                }
            }
        } else {
            trace!("No author information found");
        }

        // Sort by index
        indexed_authors.sort_by_key(|k| k.0);
        let authors: Vec<Author> = indexed_authors.into_iter().map(|(_, a)| a).collect();

        if !authors.is_empty() {
            debug!("Extracted {} authors", authors.len());
        }

        authors
    }

    /// Extracts attachment list
    #[instrument(skip(self), fields(uri = %subject))]
    fn extract_attachments(&self, subject: &NamedOrBlankNode) -> Vec<Attachment> {
        let mut attachments = Vec::new();

        // Find all attachments linked via link:link
        for triple in self.graph.triples_for_subject(subject) {
            if triple.predicate == vocab::LINK_LINK.as_ref() {
                // Get the attachment's URI
                if let oxrdf::TermRef::NamedNode(nn) = &triple.object {
                    // Directly use NamedNode to create NamedOrBlankNode
                    let attachment_subject = NamedOrBlankNode::from(*nn);
                    if let Some(attachment) =
                        self.extract_attachment_from_subject(&attachment_subject)
                    {
                        trace!(
                            "Extracted attachment: {}",
                            attachment.title.as_deref().unwrap_or("(untitled)")
                        );
                        attachments.push(attachment);
                    }
                } else if let oxrdf::TermRef::BlankNode(bn) = &triple.object {
                    // Attachment might be a BlankNode
                    let attachment_subject = NamedOrBlankNode::from(*bn);
                    if let Some(attachment) =
                        self.extract_attachment_from_subject(&attachment_subject)
                    {
                        attachments.push(attachment);
                    }
                }
            }
        }

        attachments
    }

    /// Extracts tag list from dc:subject predicates
    ///
    /// Zotero stores tags as dc:subject predicates, where each subject
    /// contains a z:AutomaticTag blank node with an rdf:value literal.
    ///
    /// # RDF Structure
    ///
    /// ```xml
    /// <dc:subject>
    ///    <z:AutomaticTag><rdf:value>tag name</rdf:value></z:AutomaticTag>
    /// </dc:subject>
    /// ```
    #[instrument(skip(self), fields(uri = %subject))]
    fn extract_tags(&self, subject: &NamedOrBlankNode) -> Vec<String> {
        let mut tags = Vec::new();

        // Find all dc:subject predicates for this item
        for triple in self.graph.triples_for_subject(subject) {
            if triple.predicate != vocab::DC_SUBJECT.as_ref() {
                continue;
            }

            // The object should be a BlankNode (z:AutomaticTag)
            let tag_blank_node = match &triple.object {
                oxrdf::TermRef::BlankNode(bn) => NamedOrBlankNode::from(*bn),
                _ => {
                    trace!("Skipping non-blank-node tag object: {}", triple.object);
                    continue;
                }
            };

            // Extract the rdf:value from the blank node
            if let Some(tag_value) = self.get_literal(&tag_blank_node, &vocab::RDF_VALUE) {
                trace!("Extracted tag: {}", tag_value);
                tags.push(tag_value);
            } else {
                trace!("Tag blank node has no rdf:value, skipping");
            }
        }

        if !tags.is_empty() {
            debug!("Extracted {} tags", tags.len());
        }

        tags
    }

    /// Extracts journal information from dcterms:isPartOf -> bib:Journal
    ///
    /// Zotero stores journal information in a `bib:Journal` node linked via
    /// the `dcterms:isPartOf` predicate.
    ///
    /// # RDF Structure
    ///
    /// ```xml
    /// <dcterms:isPartOf>
    ///     <bib:Journal>
    ///         <dc:title>Journal Name</dc:title>
    ///         <dcterms:alternative>J. Name</dcterms:alternative>
    ///         <prism:volume>123</prism:volume>
    ///         <prism:number>4</prism:number>
    ///     </bib:Journal>
    /// </dcterms:isPartOf>
    /// ```
    ///
    /// Or via URI reference:
    /// ```xml
    /// <dcterms:isPartOf rdf:resource="urn:issn:XXXX"/>
    /// ...
    /// <bib:Journal rdf:about="urn:issn:XXXX">
    ///     <dc:title>Journal Name</dc:title>
    /// </bib:Journal>
    /// ```
    #[instrument(skip(self), fields(uri = %subject))]
    fn extract_journal(&self, subject: &NamedOrBlankNode) -> Option<Journal> {
        // Find dcterms:isPartOf predicate
        let is_part_of_obj = self.get_object(subject, &vocab::DCTERMS_IS_PART_OF)?;

        // Get the subject of the Journal node
        let journal_subject = term_to_subject(&is_part_of_obj)?;

        // Verify it's a bib:Journal (optional check, but good for correctness)
        // Note: Some RDF exports might not have explicit rdf:type, so we still try to extract
        let is_journal = self
            .get_object(&journal_subject, &vocab::RDF_TYPE)
            .map(|t| {
                if let Term::NamedNode(nn) = t {
                    nn.as_str() == vocab::BIB_JOURNAL.as_str()
                } else {
                    false
                }
            })
            .unwrap_or(true); // If no type, assume it could be a journal

        if !is_journal {
            trace!("isPartOf target is not a bib:Journal, skipping");
            return None;
        }

        // Extract journal fields
        let title = self.get_literal(&journal_subject, &vocab::DC_TITLE);
        let number = self.get_literal(&journal_subject, &vocab::PRISM_NUMBER);
        let volume = self.get_literal(&journal_subject, &vocab::PRISM_VOLUME);

        // Only return Journal if at least one field has data
        if title.is_none() && number.is_none() && volume.is_none() {
            trace!("Journal node has no extractable fields, returning None");
            return None;
        }

        let journal = Journal {
            title,
            number,
            volume,
        };

        debug!(
            "Extracted journal: {:?}",
            journal.title.as_deref().unwrap_or("(no title)")
        );

        Some(journal)
    }

    /// Extracts attachment information from a NamedOrBlankNode
    fn extract_attachment_from_subject(&self, subject: &NamedOrBlankNode) -> Option<Attachment> {
        let uri = subject.to_string();
        let title = self.get_literal(subject, &vocab::DC_TITLE);
        let content_type = self.get_literal(subject, &vocab::LINK_TYPE);

        // Extract local file path from z:file predicate (URI reference, not literal)
        let path = self.get_uri(subject, &vocab::Z_FILE).and_then(|uri| {
            // Extract the path portion from the URI and decode percent-encoding
            extract_path_from_file_uri(&uri)
        });

        // Extract URL from dc:identifier
        let url = self.extract_attachment_url(subject);

        Some(Attachment {
            uri,
            title,
            content_type,
            path,
            url,
        })
    }

    /// Extracts attachment URL from dc:identifier
    fn extract_attachment_url(&self, subject: &NamedOrBlankNode) -> Option<String> {
        // dc:identifier may contain a dcterms:URI node
        if let Some(identifier_obj) = self.get_object(subject, &vocab::DC_IDENTIFIER) {
            match &identifier_obj {
                Term::BlankNode(bn) => {
                    // Find rdf:value in dcterms:URI node
                    let subject = NamedOrBlankNode::from(bn.clone());
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

    fn extract_person(&self, subject: &NamedOrBlankNode) -> Option<Author> {
        let surname = self.get_literal(subject, &vocab::FOAF_SURNAME);
        let given = self.get_literal(subject, &vocab::FOAF_GIVENNAME);

        // At least one field is required for a valid author
        if surname.is_none() && given.is_none() {
            warn!("Author node missing name information: {}", subject);
            return None;
        }

        Some(Author {
            surname,
            given_name: given,
            full_name: None,
        })
    }

    /// Gets object term
    fn get_object(&self, subject: &NamedOrBlankNode, predicate: &oxrdf::NamedNode) -> Option<Term> {
        self.graph
            .object_for_subject_predicate(subject, predicate)
            .map(|t| t.into_owned())
    }

    /// Gets literal value
    fn get_literal(
        &self,
        subject: &NamedOrBlankNode,
        predicate: &oxrdf::NamedNode,
    ) -> Option<String> {
        self.get_object(subject, predicate).and_then(|obj| {
            if let Term::Literal(lit) = obj {
                Some(lit.value().to_string())
            } else {
                None
            }
        })
    }

    /// Gets URI value (NamedNode)
    fn get_uri(&self, subject: &NamedOrBlankNode, predicate: &oxrdf::NamedNode) -> Option<String> {
        self.get_object(subject, predicate).and_then(|obj| {
            if let Term::NamedNode(node) = obj {
                Some(node.to_string())
            } else {
                None
            }
        })
    }

    /// Checks if a term is of rdf:Seq type
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

/// Extracts NamedOrBlankNode from Term (only supports BlankNode and NamedNode)
fn term_to_subject(term: &Term) -> Option<NamedOrBlankNode> {
    match term {
        Term::BlankNode(bn) => Some(NamedOrBlankNode::from(bn.clone())),
        Term::NamedNode(nn) => Some(NamedOrBlankNode::from(nn.clone())),
        _ => None,
    }
}

/// Extracts DOI from URI
///
/// Supported formats:
/// - https://doi.org/10.xxx/yyy
/// - http://dx.doi.org/10.xxx/yyy
fn extract_doi_from_uri(uri: &str) -> Option<String> {
    // Remove RDF angle brackets
    let uri = uri.trim_start_matches('<').trim_end_matches('>');

    // Try to extract from doi.org URI
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

/// Extracts local file path from zotero file URI
///
/// Zotero stores file paths as URIs like:
/// - `http://zotero.org/files/2469/paper.pdf`
/// - `http://zotero.org/files/2478/Kihl%20and%20K%C3%A4llberg.pdf`
///
/// This function extracts the relative path portion and decodes percent-encoding.
/// Returns path in format: `files/2469/paper.pdf`
fn extract_path_from_file_uri(uri: &str) -> Option<String> {
    // Remove RDF angle brackets if present
    let uri = uri.trim_start_matches('<').trim_end_matches('>');

    // Extract path from zotero.org/files/ URI
    let path = if let Some(rest) = uri.strip_prefix("http://zotero.org/files/") {
        // Prepend "files/" to maintain the expected format
        format!("files/{}", rest)
    } else {
        // For other URIs, try to extract path after the last segment that looks like a directory
        return None;
    };

    // Decode percent-encoding (e.g., %20 -> space, %C3%A4 -> ä)
    match percent_decode(&path) {
        Ok(decoded) => Some(decoded),
        Err(e) => {
            warn!("Failed to decode path '{}': {}", path, e);
            Some(path) // Fallback to raw path
        }
    }
}

/// Decodes percent-encoded string
fn percent_decode(s: &str) -> Result<String, std::string::FromUtf8Error> {
    let mut bytes = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Try to parse the next two characters as hex
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    bytes.push(byte);
                    continue;
                }
            }
            // If parsing fails, push '%' and continue
            bytes.push(b'%');
            for c in hex.chars() {
                bytes.push(c as u8);
            }
        } else {
            // Handle UTF-8 characters
            let mut buf = [0u8; 4];
            for byte in c.encode_utf8(&mut buf).as_bytes() {
                bytes.push(*byte);
            }
        }
    }

    String::from_utf8(bytes)
}
