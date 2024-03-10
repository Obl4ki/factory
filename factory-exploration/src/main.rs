#![allow(unused)]
use std::path::PathBuf;

use anyhow::Result;

use factory_lib::{prelude::*, traits::DataSource};
use itertools::Itertools as _;

fn main() -> Result<()> {
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

    let mut data = DataSet::from_file("recipe-lister/recipe.json", &natural_items)?;

    let recipe_filters = [
        "iron-plate",
        "copper-plate",
        "copper-cable",
        "electronic-circuit",
        "basic-oil-processing",
        "advanced-oil-processing",
        "plastic-bar",
        "advanced-circuit",
        "empty-crude-oil-barrel",
        "fill-crude-oil-barrel",
        "empty-petroleum-gas-barrel",
        "fill-petroleum-gas-barrel",
        "empty-crude-oil-barrel",
        "fill-crude-oil-barrel",
        "empty-crude-oil-barrel",
        "fill-crude-oil-barrel",
        "empty-barrel",
        "steel-plate",
    ]
    .into_iter()
    .map(|recipe_name| data.get_recipe(recipe_name))
    .collect_vec();

    data.recipes = data
        .iter_recipes()
        .filter(|recipe| recipe_filters.contains(recipe))
        .cloned()
        .collect();

    let mut recipe_graph = CraftingGraph::from_dataset(&data);

    if save_figures {
        let file_name: PathBuf = "outputs/explore.svg".into();
        recipe_graph.save_as_svg(file_name)?;
    }

    Ok(())
}
