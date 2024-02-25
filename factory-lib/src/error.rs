use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Json file doesn't exist: `{0}`")]
    JsonFileNotFound(io::Error),

    #[error("Failed to parse provided json file `{0}`")]
    BadJson(serde_json::Error),
}
