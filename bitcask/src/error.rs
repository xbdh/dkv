use std::result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum errors {
    #[error("Failed to read from file")]
    FailedToWriteFromFile,
    #[error("Failed to write to file")]
    FailedTosyncFromFile,
    #[error("Failed to sync to file")]
    FailedToReadFromFile,
    #[error("Failed to open file")]
    FialedToOpenFile,
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    #[error("Parse error: {0}")]

    ParseError(std::num::ParseIntError),
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T>=std::result::Result<T, errors>;