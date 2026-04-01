use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Daemon error: {0}")]
    Daemon(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Http(e.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for AppError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        AppError::WebSocket(e.to_string())
    }
}
