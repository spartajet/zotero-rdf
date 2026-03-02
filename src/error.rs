use thiserror::Error;

/// 解析过程中的位置信息
#[derive(Debug, Clone, Default)]
pub struct ErrorLocation {
    /// 字节偏移量
    pub byte_offset: Option<usize>,
    /// 行号 (1-based)
    pub line: Option<usize>,
    /// 列号 (1-based)
    pub column: Option<usize>,
}

impl ErrorLocation {
    /// 创建未知位置
    pub fn unknown() -> Self {
        Self::default()
    }

    /// 创建指定行列的位置
    pub fn at(line: usize, column: usize) -> Self {
        Self {
            byte_offset: None,
            line: Some(line),
            column: Some(column),
        }
    }

    /// 是否有位置信息
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

/// Zotero RDF 解析错误类型
#[derive(Error, Debug)]
pub enum ZoteroRdfError {
    /// IO 错误（文件不存在、权限问题等）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 无效的 URI/IRI
    #[error("Invalid URI: {uri}")]
    InvalidUri {
        /// 无效的 URI 字符串
        uri: String,
    },

    /// RDF/XML 解析错误
    #[error("RDF/XML parse error at {location}: {message}")]
    ParseError {
        /// 错误消息
        message: String,
        /// 错误位置
        location: ErrorLocation,
    },

    /// 字符编码错误
    #[error("Encoding error at {location}: {message}")]
    EncodingError {
        /// 错误消息
        message: String,
        /// 错误位置
        location: ErrorLocation,
    },

    /// 缺少必需字段
    #[error("Missing required field '{field}' in {context}")]
    MissingField {
        /// 缺少的字段名
        field: String,
        /// 上下文描述
        context: String,
    },

    /// 不支持的特性
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

impl ZoteroRdfError {
    /// 创建简单的解析错误（无位置信息）
    pub fn parse_error(message: impl Into<String>) -> Self {
        ZoteroRdfError::ParseError {
            message: message.into(),
            location: ErrorLocation::unknown(),
        }
    }

    /// 创建带位置信息的解析错误
    pub fn parse_error_at(message: impl Into<String>, line: usize, column: usize) -> Self {
        ZoteroRdfError::ParseError {
            message: message.into(),
            location: ErrorLocation::at(line, column),
        }
    }

    /// 创建简单的编码错误
    pub fn encoding_error(message: impl Into<String>) -> Self {
        ZoteroRdfError::EncodingError {
            message: message.into(),
            location: ErrorLocation::unknown(),
        }
    }

    /// 创建缺少字段错误
    pub fn missing_field(field: impl Into<String>, context: impl Into<String>) -> Self {
        ZoteroRdfError::MissingField {
            field: field.into(),
            context: context.into(),
        }
    }
}

/// 解析统计信息
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// 成功解析的三元组数量
    pub triples_count: usize,
    /// 错误数量
    pub error_count: usize,
    /// 警告数量
    pub warning_count: usize,
}

/// 解析选项，控制错误处理行为
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// 最大允许的错误数量，超过则返回错误
    pub max_errors: usize,
    /// 是否在遇到错误时继续解析
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
    /// 创建严格模式（遇到错误立即停止）
    pub fn strict() -> Self {
        Self {
            max_errors: 1,
            continue_on_error: false,
        }
    }

    /// 创建宽松模式（尽可能容忍错误）
    pub fn lenient() -> Self {
        Self {
            max_errors: usize::MAX,
            continue_on_error: true,
        }
    }
}
