use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZoteroRdfError {
    #[error("Failed to read file or stream: {0}")]
    Io(#[from] std::io::Error),

    #[error("RDF/XML parsing failed: {0}")]
    ParseError(String),

    #[error("Invalid URI encountered: {0}")]
    InvalidUri(String),
}
