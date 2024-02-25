use factory_lib::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
enum ApplicationError {
    #[error(transparent)]
    DataError(#[from] DataError),
}

fn main() -> Result<(), ApplicationError> {
    let data = load_dataset("recipe-lister/recipe.json")?;
    println!("{data:#?}");
    // get_crafting_path();
    Ok(())
}
