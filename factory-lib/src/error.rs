use std::io;
use thiserror::Error;

pub type FactoryResult<T> = Result<T, FactoryError>;

#[derive(Error, Debug)]
pub enum FactoryError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Failed to parse provided json file `{0}`")]
    BadJson(serde_json::Error),

    #[error("Error when spawning command: `{0}`")]
    CommandSpawn(String),

    #[error("Failed to interpret the output of command")]
    CommandOutputError(#[from] std::str::Utf8Error)
}
