use crate::error::DataError;
use crate::domain::RecipeJsonData;
use std::{fs, path::Path};

pub mod domain;
pub mod error;
pub mod prelude;


pub fn load(json_path: impl AsRef<Path>) -> Result<Vec<RecipeJsonData>, error::DataError> {
    let recipes_str = fs::read_to_string(json_path).map_err(DataError::JsonFileNotFound)?;

    serde_json::from_str(&recipes_str).map_err(DataError::BadJson)
}
