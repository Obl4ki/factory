use std::path::PathBuf;

use common::AppResult;
use factory_lib::data::DataSet;
use factory_lib::domain::CraftingGraph;
use factory_lib::traits::DataSource as _;

mod common;
mod error;

fn main() -> AppResult<()> {
    let _save_figures = true;

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

    println!("Reading dataset");

    let data = DataSet::from_file("recipe-lister/recipe.json", &natural_items)?;

    println!("Parsing crafting graph");
    let recipe_graph = CraftingGraph::from_dataset(&data);

    // println!("Saving crafting graph to file");

    // if save_figures {
    //     let file_name: PathBuf = "outputs/explore.svg".into();
    //     recipe_graph.save_as_svg(file_name)?;
    // }

    println!("Generating crafting possibilities");

    let crafting_possibilities = recipe_graph
        .get_crafting_trees(recipe_graph.get_item_node("utility-science-pack"), 200)
        .expect("Should be ok");

    for (idx, possibility) in crafting_possibilities.into_iter().enumerate().rev() {
        println!("{}. with {} nodes", idx + 1, possibility.data.node_count());
        // let file_name: PathBuf = format!("outputs/{idx}.svg").into();
        // possibility.save_as_svg(file_name)?;
    }

    Ok(())
}
