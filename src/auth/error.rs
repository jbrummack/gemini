#[derive(Debug, thiserror::Error)]
pub enum NetConnError {
    #[error("{0}")]
    UnexpectedResponse(String),
    #[error("{0}")]
    InvalidChunk(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Unable to decode because of {0}")]
    Des(#[from] serde_json::Error),
    #[error("{0}")]
    Jwt(#[from] JwtError),
    #[error("{0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("{0}")]
    InvalidUri(String),
    #[error("Hyper error: {0}")]
    HyperClient(#[from] hyper_util::client::legacy::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::http::Error),
}
#[derive(Debug, thiserror::Error)]
pub enum UacError {
    #[error("Unable to open \"{0}\" because of {1}")]
    Io(String, std::io::Error),
    #[error("Unable to decode because of {0}")]
    Des(#[from] serde_json::Error),
}
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("{0}")]
    Time(#[from] std::time::SystemTimeError),
    #[error("{0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Unable to decode because of {0}")]
    Des(#[from] serde_json::Error),
}
