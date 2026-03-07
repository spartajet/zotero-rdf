//! Zotero data models
//!
//! This module defines the data structures for Zotero items:
//! - [`ZoteroItem`]: Represents a bibliographic item (journal article, book, etc.)
//! - [`Author`]: Represents author information
//! - [`Attachment`]: Represents attachment information (PDFs, etc.)

use serde::{Deserialize, Serialize};

/// Represents a Zotero item (journal article, book, etc.)
///
/// `ZoteroItem` contains structured data extracted from the RDF graph,
/// including the main metadata of a bibliographic item.
///
/// # Fields
///
/// * `uri` - Unique identifier of the item (Subject URI in RDF)
/// * `item_type` - Zotero item type (e.g., journalArticle, book, thesis)
/// * `title` - Title
/// * `authors` - List of authors, in original order
/// * `date` - Publication date
/// * `doi` - DOI identifier
/// * `abstract_note` - Abstract
/// * `attachments` - List of attachments (PDFs, etc.)
/// * `tags` - List of tags (from dc:subject/z:AutomaticTag)
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
///     tags: vec!["research".to_string(), "paper".to_string()],
/// };
///
/// assert_eq!(item.authors[0].display_name(), "Doe, John");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoteroItem {
    /// Item URI (primary key)
    ///
    /// In Zotero RDF, this is the unique identifier of the item,
    /// typically in the format `http://zotero.org/export#item_XXX`
    pub uri: String,

    /// Zotero item type
    ///
    /// Common types include:
    /// - `journalArticle` - Journal articles
    /// - `book` - Books
    /// - `thesis` - Theses and dissertations
    /// - `conferencePaper` - Conference papers
    /// - `report` - Reports
    pub item_type: String,

    /// Title
    pub title: Option<String>,

    /// List of authors (in original order)
    ///
    /// Author order matches the order in the Zotero library,
    /// achieved by parsing the `rdf:Seq` structure in the RDF.
    pub authors: Vec<Author>,

    /// Publication date
    ///
    /// Date format may be a year (e.g., `2024`) or a full date (e.g., `2024-01-15`).
    pub date: Option<String>,

    /// DOI (Digital Object Identifier)
    ///
    /// The DOI may be extracted directly from the `bibo:doi` predicate,
    /// or parsed from the item URI (e.g., `https://doi.org/10.xxx/yyy`).
    pub doi: Option<String>,

    /// Abstract
    pub abstract_note: Option<String>,

    /// List of attachments (PDFs, etc.)
    ///
    /// Attachments are linked to the main item via the `link:link` predicate.
    /// An item may have multiple attachments (e.g., PDF, HTML snapshot).
    pub attachments: Vec<Attachment>,

    /// List of tags
    ///
    /// Tags are extracted from `dc:subject` predicates,
    /// where each subject contains a `z:AutomaticTag` blank node with an `rdf:value` literal.
    pub tags: Vec<String>,
}

/// Represents author information
///
/// Author information is extracted from the FOAF ontology,
/// including surname and given name.
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
    /// Given name (first name)
    ///
    /// Extracted from `foaf:givenName`.
    pub given_name: Option<String>,

    /// Surname (last name)
    ///
    /// Extracted from `foaf:surname`.
    pub surname: Option<String>,

    /// Full name
    ///
    /// In some cases, Zotero may provide the complete name directly.
    pub full_name: Option<String>,
}

impl Author {
    /// Generates a name in standard citation format
    ///
    /// Format is "Surname, GivenName" (e.g., "Smith, Jane").
    /// If only surname or given name exists, returns that one.
    /// If neither exists, returns `full_name` or an empty string.
    ///
    /// # Example
    ///
    /// ```
    /// use zotero_rdf::Author;
    ///
    /// let author = Author {
    ///     surname: Some("Li".to_string()),
    ///     given_name: Some("Ming".to_string()),
    ///     full_name: None,
    /// };
    ///
    /// assert_eq!(author.display_name(), "Li, Ming");
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

/// Represents attachment information (PDFs, etc.)
///
/// Attachments are linked to the main item via the `link:link` predicate.
/// A Zotero item may have multiple attachments, such as:
/// - PDF full text
/// - HTML webpage snapshot
/// - Plain text notes
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
///     path: Some("files/2474/paper.pdf".to_string()),
///     url: Some("https://example.com/paper.pdf".to_string()),
/// };
///
/// assert_eq!(attachment.content_type, Some("application/pdf".to_string()));
/// assert_eq!(attachment.path, Some("files/2474/paper.pdf".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment URI
    ///
    /// In RDF, this is the unique identifier of the attachment node.
    pub uri: String,

    /// Attachment title (usually the filename)
    ///
    /// Example: "Doe et al_2024_Research Paper.pdf"
    pub title: Option<String>,

    /// Attachment type (MIME type)
    ///
    /// Common types:
    /// - `application/pdf` - PDF files
    /// - `text/html` - HTML webpages
    /// - `text/plain` - Plain text
    pub content_type: Option<String>,

    /// Local file path (relative or absolute)
    ///
    /// Corresponds to `rdf:resource` in Zotero RDF exports.
    /// This is the path where the attachment file is stored locally.
    ///
    /// Example: `files/2474/paper.pdf`
    pub path: Option<String>,

    /// Original URL (from dc:identifier)
    ///
    /// The original web URL where the file was downloaded from.
    /// May be `None` for locally created attachments.
    ///
    /// Example: `https://example.com/paper.pdf`
    pub url: Option<String>,
}
