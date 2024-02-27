use std::collections::HashMap;

use crate::entities::Item;
use crate::entities::ItemAmount;
use crate::entities::Recipe;

use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use petgraph::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node<'a> {
    Item(&'a Item),
    Recipe(&'a Recipe),
}

/// Create a directed graph of items and recipes.
/// Each node is either item or recipe, which alternate between one another. In other words, there
/// is no node which has any neighbour with the same type as itself.
/// This works because:
/// - Item can be made from several recipes (Recipe -> Item)
/// - Item can be used in several recipes (Item -> Recipe)
/// - Recipe can use several items (Item -> Recipe)
/// - Recipe can return several items (Recipe -> Item)
///
/// And such, if the edge is
/// - (Recipe -> Item), then it has a weight that describes how much items can be crafted from this recipe.
/// - (Item -> Recipe), then it's weight describes the amount of items needed for target recipe.
pub fn create_crafting_tree(dataset: &[Recipe]) -> DiGraph<Node, ItemAmount> {
    let mut nodes: HashMap<Node, NodeIndex> = HashMap::new();
    let mut g = DiGraph::new();

    for recipe in dataset {
        let recipe_idx = g.add_node(Node::Recipe(recipe));

        for (amount, item) in &recipe.result {
            // ensure that all results item are inserted into graph before creating relevant edges
            nodes
                .entry(Node::Item(item))
                .or_insert_with(|| g.add_node(Node::Item(item)));

            let item_idx = nodes[&Node::Item(item)];

            // Add edge from item to recipe with amount of crafted items
            g.add_edge(recipe_idx, item_idx, *amount);
        }

        for (amount, item) in &recipe.ingredients {
            nodes
                .entry(Node::Item(item))
                .or_insert_with(|| g.add_node(Node::Item(item)));

            let item_idx = nodes[&Node::Item(item)];

            g.add_edge(item_idx, recipe_idx, *amount);
        }
    }

    g
}
