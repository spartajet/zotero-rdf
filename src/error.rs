use thiserror::Error;

/// Location information during parsing
#[derive(Debug, Clone, Default)]
pub struct ErrorLocation {
    /// Byte offset
    pub byte_offset: Option<usize>,
    /// Line number (1-based)
    pub line: Option<usize>,
    /// Column number (1-based)
    pub column: Option<usize>,
}

impl ErrorLocation {
    /// Creates an unknown location
    pub fn unknown() -> Self {
        Self::default()
    }

    /// Creates a location at a specific line and column
    pub fn at(line: usize, column: usize) -> Self {
        Self {
            byte_offset: None,
            line: Some(line),
            column: Some(column),
        }
    }

    /// Returns whether there is location information
    pub fn has_location(&self) -> bool {
        self.line.is_some() || self.column.is_some() || self.byte_offset.is_some()
    }
}

impl std::fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(col)) => write!(f, "line {}, column {}", line, col),
            (Some(line), None) => write!(f, "line {}", line),
            (None, Some(col)) => write!(f, "column {}", col),
            (None, None) => {
                if let Some(offset) = self.byte_offset {
                    write!(f, "byte {}", offset)
                } else {
                    write!(f, "unknown position")
                }
            }
        }
    }
}

/// Zotero RDF parsing error types
#[derive(Error, Debug)]
pub enum ZoteroRdfError {
    /// IO error (file not found, permission issues, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid URI/IRI
    #[error("Invalid URI: {uri}")]
    InvalidUri {
        /// The invalid URI string
        uri: String,
    },

    /// RDF/XML parsing error
    #[error("RDF/XML parse error at {location}: {message}")]
    ParseError {
        /// Error message
        message: String,
        /// Error location
        location: ErrorLocation,
    },

    /// Character encoding error
    #[error("Encoding error at {location}: {message}")]
    EncodingError {
        /// Error message
        message: String,
        /// Error location
        location: ErrorLocation,
    },

    /// Missing required field
    #[error("Missing required field '{field}' in {context}")]
    MissingField {
        /// Name of the missing field
        field: String,
        /// Context description
        context: String,
    },

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

impl ZoteroRdfError {
    /// Creates a simple parse error (without location information)
    pub fn parse_error(message: impl Into<String>) -> Self {
        ZoteroRdfError::ParseError {
            message: message.into(),
            location: ErrorLocation::unknown(),
        }
    }

    /// Creates a parse error with location information
    pub fn parse_error_at(message: impl Into<String>, line: usize, column: usize) -> Self {
        ZoteroRdfError::ParseError {
            message: message.into(),
            location: ErrorLocation::at(line, column),
        }
    }

    /// Creates a simple encoding error
    pub fn encoding_error(message: impl Into<String>) -> Self {
        ZoteroRdfError::EncodingError {
            message: message.into(),
            location: ErrorLocation::unknown(),
        }
    }

    /// Creates a missing field error
    pub fn missing_field(field: impl Into<String>, context: impl Into<String>) -> Self {
        ZoteroRdfError::MissingField {
            field: field.into(),
            context: context.into(),
        }
    }
}

/// Parsing statistics
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// Number of successfully parsed triples
    pub triples_count: usize,
    /// Number of errors
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
}

/// Parse options controlling error handling behavior
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Maximum allowed number of errors, after which an error is returned
    pub max_errors: usize,
    /// Whether to continue parsing when errors are encountered
    pub continue_on_error: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_errors: 100,
            continue_on_error: true,
        }
    }
}

impl ParseOptions {
    /// Creates strict mode (stops immediately on error)
    pub fn strict() -> Self {
        Self {
            max_errors: 1,
            continue_on_error: false,
        }
    }

    /// Creates lenient mode (tolerates as many errors as possible)
    pub fn lenient() -> Self {
        Self {
            max_errors: usize::MAX,
            continue_on_error: true,
        }
    }
}
