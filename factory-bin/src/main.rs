use std::path::PathBuf;

use common::AppResult;

use factory_lib::domain::Node;
use factory_lib::entities::Item;
use factory_lib::prelude::*;

mod common;
mod error;

fn main() -> AppResult<()> {
    let max_number_of_results = usize::MAX;

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

    let data = load_dataset("recipe-lister/recipe.json", &natural_items)?;

    let recipe_graph = CraftingGraph::from_dataset(&data);

    let search_item = &Item {
        name: "electronic-circuit".to_string(),
        natural: false,
    };

    let mut graphs = recipe_graph
        .get_crafting_trees(Node::Item(search_item), max_number_of_results)
        .expect("Result should be present");

    println!("Total number of graphs: {}", graphs.len());

    graphs.sort_by(|graph1, graph2| graph1.data.node_count().cmp(&graph2.data.node_count()));

    for (idx, crafting_possibility) in graphs.iter().take(3).enumerate() {
        println!(
            "Recipe path {idx} created with {} nodes",
            crafting_possibility.data.node_count()
        );

        let file_name: PathBuf = format!("outputs/output_{idx}.svg").into();
        crafting_possibility.save_as_svg(file_name, true)?;
    }

    Ok(())
}
