use anyhow::Error;

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("parse id error {0}")]
    ParseIdError(#[from] uuid::Error),

    #[error("parse response error {0}")]
    ParseResponseError(Error),

    #[error("io error {0}")]
    IOError(#[from] std::io::Error),

    #[error("not found")]
    NotFound,

    #[error("internal error {0}")]
    InternalError(#[from] Error),

    #[error("unexpected response {0}")]
    UnexpectedResponse(String),

    #[error("worker error {0}")]
    WorkerError(Error),
}
