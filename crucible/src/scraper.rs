#[derive(Debug, thiserror::Error)]
pub enum ScrapeError {
    #[error("Network error: {0}")]
    NetworkError(reqwest::Error),
    #[error("Parse error: {0}")]
    ParseError(serde_json::Error),
    #[error("API error: {0}")]
    APIError(String),
}
