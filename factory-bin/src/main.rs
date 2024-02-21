use thiserror::Error;
use factory_io::prelude::*;

#[derive(Error, Debug)]
enum ApplicationError {
    #[error(transparent)]
    DataError(#[from] DataError),
}

fn main() -> Result<(), ApplicationError> {
    let data = load("recipes.json")?;

    println!("{data:#?}");
    Ok(())
}
