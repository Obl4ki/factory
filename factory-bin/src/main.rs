use factory_lib::{domain::Node, entities::Item, prelude::*};

use thiserror::Error;

#[derive(Error, Debug)]
enum ApplicationError {
    #[error(transparent)]
    DataError(#[from] DataError),
}

fn main() -> Result<(), ApplicationError> {
    let data = load_dataset("recipe-lister/recipe.json")?;
    let recipe_graph = CraftingGraph::from_dataset(&data);
    Ok(())
}
