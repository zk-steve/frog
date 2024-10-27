use anyhow::Error;

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("parse int error {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("io error {0}")]
    IOError(#[from] std::io::Error),

    #[error("missing parameters")]
    MissingParameters,

    #[error("not found")]
    NotFound,

    #[error("internal error {0}")]
    InternalError(#[from] Error),

    #[error("unexpected response {0}")]
    UnexpectedResponse(String),
}
