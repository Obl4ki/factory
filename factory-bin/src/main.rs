use factory_lib::{domain::Node, entities::Item, prelude::*};

use petgraph::dot::{Config, Dot};
use thiserror::Error;

#[derive(Error, Debug)]
enum ApplicationError {
    #[error(transparent)]
    DataError(#[from] DataError),
}

#[allow(unused)]
fn main() -> Result<(), ApplicationError> {
    let data = load_dataset("recipe-lister/recipe.json")?;
    let recipe_graph = CraftingGraph::from_dataset(&data);

    let search_item = &Item {
        name: "electronic-circuit".to_string(),
    };
    
    let graphs = recipe_graph
        .get_crafting_trees(Node::Item(search_item))
        .expect("Result should be present");

    Ok(())
}
