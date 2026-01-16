//! Error types for the Extended SDK.

use thiserror::Error;

/// Result type alias for Extended SDK operations.
pub type Result<T> = std::result::Result<T, ExtendedError>;

/// Main error type for the Extended SDK.
#[derive(Error, Debug)]
pub enum ExtendedError {
    /// HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error {code}: {message}")]
    Api {
        /// Error code from the API (can be numeric or string like "NOT_FOUND").
        code: String,
        /// Error message from the API.
        message: String,
    },

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// URL parsing error.
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Signing/cryptographic error.
    #[error("Signing error: {0}")]
    Signing(String),

    /// Invalid parameter provided.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Order validation error.
    #[error("Order validation error: {0}")]
    OrderValidation(String),
}

/// API error response structure from Extended Exchange.
#[derive(Debug, serde::Deserialize)]
pub struct ApiErrorResponse {
    pub status: String,
    pub error: ApiErrorDetail,
}

/// Error code that can be either a number or a string.
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum ErrorCode {
    Numeric(i32),
    Text(String),
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Numeric(n) => write!(f, "{}", n),
            ErrorCode::Text(s) => write!(f, "{}", s),
        }
    }
}

/// Detail of an API error.
#[derive(Debug, serde::Deserialize)]
pub struct ApiErrorDetail {
    pub code: ErrorCode,
    pub message: String,
}

impl From<ApiErrorResponse> for ExtendedError {
    fn from(resp: ApiErrorResponse) -> Self {
        ExtendedError::from_api_error(resp.error.code, resp.error.message)
    }
}

/// Map common API error codes to specific error types.
impl ExtendedError {
    /// Create an API error from code and message, mapping to specific variants where applicable.
    pub fn from_api_error(code: ErrorCode, message: String) -> Self {
        match &code {
            ErrorCode::Numeric(n) => match n {
                // Rate limit errors
                429 => ExtendedError::RateLimitExceeded,
                // Authentication errors (1100-1102)
                1100..=1102 => ExtendedError::Authentication(message),
                // Order validation errors (1120-1148)
                1120..=1148 => ExtendedError::OrderValidation(message),
                // Generic API error
                _ => ExtendedError::Api { code: code.to_string(), message },
            },
            ErrorCode::Text(_) => ExtendedError::Api { code: code.to_string(), message },
        }
    }
}
