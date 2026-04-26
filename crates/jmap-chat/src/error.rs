#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("authentication failed: HTTP {0}")]
    AuthFailed(u16),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("invalid session: {0}")]
    InvalidSession(&'static str),

    #[error("method not found in response: {0}")]
    MethodNotFound(String),

    #[error("JMAP method error: {error_type}: {description}")]
    MethodError {
        error_type: String,
        description: String,
    },

    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("SSE frame too large")]
    SseFrameTooLarge,
}
