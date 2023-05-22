use std::convert::From;
use thiserror;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Parameter not defined: {0}")]
    NotDefined(String),
    #[error("Plot axis not defined: {0}")]
    AxisNotDefined(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::FileNotFound(err.to_string())
    }
}
