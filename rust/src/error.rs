use std::convert::From;
use thiserror;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Generic(String),
    #[error("{0}")]
    ParseError(String),
    #[error("Parameter not defined: {0}")]
    NotDefined(String),
    #[error("Plot axis not defined: {0}")]
    AxisNotDefined(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Index not found: {0}")]
    IndexError(String),
    #[error("Image suffix not supported: {0}")]
    InvalidImageSuffix(String),
    #[error("Unable to open file for reading: {0}")]
    ReaderError(String),
    #[error("Unable to process JSON: {0}")]
    SerdeError(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::FileNotFound(err.to_string())
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Error::ReaderError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeError(err.to_string())
    }
}
