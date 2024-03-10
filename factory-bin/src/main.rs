use std::path::PathBuf;

use common::AppResult;
use factory_lib::{data::DataSet, domain::CraftingGraph, traits::DataSource as _};

mod common;
mod error;

fn main() -> AppResult<()> {
    let save_figures = true;

    let natural_items: Vec<String> = [
        "coal",
        "copper-ore",
        "crude-oil",
        "iron-ore",
        "raw-fish",
        "stone",
        "uranium-ore",
        "used-up-uranium-fuel-cell",
        "water",
        "wood",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect();

    let data = DataSet::from_file("recipe-lister/recipe.json", &natural_items)?;
    let recipe_graph = CraftingGraph::from_dataset(&data);

    if save_figures {
        let file_name: PathBuf = "outputs/explore.svg".into();
        recipe_graph.save_as_svg(file_name)?;
    }
    Ok(())
}
