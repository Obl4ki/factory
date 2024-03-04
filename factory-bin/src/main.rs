use std::io::Write as _;
use std::process::{Command, Stdio};
use std::{fs, str};

use std::path::PathBuf;

use common::AppResult;
use error::AppError;
use factory_lib::domain::Node;
use factory_lib::entities::Item;
use factory_lib::prelude::*;

mod common;
mod error;

fn main() -> AppResult<()> {
    let max_number_of_results = 3;

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
        name: "spidertron".to_string(),
        natural: false,
    };

    let mut graphs = recipe_graph
        .get_crafting_trees(Node::Item(search_item), max_number_of_results)
        .expect("Result should be present");

    println!("Total number of graphs: {}", graphs.len());

    graphs.sort_by(|graph1, graph2| graph1.data.node_count().cmp(&graph2.data.node_count()));

    for (idx, crafting_possibility) in graphs.iter().enumerate() {
        println!(
            "Recipe path {idx} created with {} nodes",
            crafting_possibility.data.node_count()
        );

        let file_name: PathBuf = format!("outputs/output_{idx}.svg").into();
        graph_to_svg_file(crafting_possibility, file_name)?;
    }

    Ok(())
}

fn graph_to_svg_file(graph: &CraftingGraph<'_>, file_name: PathBuf) -> AppResult<()> {
    let dot = graph.to_dot();
    let mut cmd = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let mut stdin = cmd.stdin.take().ok_or(AppError::StdioPipe)?;
        stdin.write_all(dot.as_bytes())?;
    }

    let output = cmd.wait_with_output()?;

    // if !output.stdout.is_empty() {
    //     println!("stdout: {}", str::from_utf8(&output.stdout)?);
    // }

    if !output.stderr.is_empty() {
        println!("stderr: {}", str::from_utf8(&output.stderr)?);
    }

    let mut file = fs::File::create(file_name)?;
    file.write_all(&output.stdout)?;

    Ok(())
}
