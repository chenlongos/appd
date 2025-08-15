#[derive(Debug, thiserror::Error)]
pub enum DError {
    #[error("Unknown error occurred: {0}")]
    Unknown(&'static str),
    #[error("Operation timed out")]
    Timeout,
    #[error("No memory available")]
    NoMemory,
    #[error("Invalid parameter")]
    InvalidParameter,
}
