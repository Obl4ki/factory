use std::collections::HashMap;

use crate::entities::Item;
use crate::entities::ItemAmount;
use crate::entities::Recipe;

use petgraph::graph::NodeIndex;
use petgraph::prelude::*;

#[derive(Debug, Clone)]
pub struct CraftingGraph<'data> {
    data: DiGraph<Node<'data>, ItemAmount>,
}

impl<'data> From<DiGraph<Node<'data>, ItemAmount>> for CraftingGraph<'data> {
    fn from(graph: DiGraph<Node<'data>, ItemAmount>) -> Self {
        Self { data: graph }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
impl<'data> CraftingGraph<'data> {
    pub fn from_dataset(dataset: &'data [Recipe]) -> Self {
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

        g.into()
    }

    /// Get all indices of item nodes that are direct input items to the recipe provided.
    /// If the node is not a recipe or it doesn't exist in graph, None is returned.
    pub fn get_recipes_input_idxs(&self, node: Node) -> Option<Vec<NodeIndex>> {
        match node {
            Node::Recipe(_) => Some(
                self.data
                    .neighbors_directed(self.get_node_idx(node)?, Direction::Incoming)
                    .collect(),
            ),
            Node::Item(_) => None,
        }
    }

    /// Get all indices of item nodes that are direct output items to the recipe provided.
    /// If the node is not a recipe or it doesn't exist in graph, None is returned.
    pub fn get_recipes_output_idxs(&self, node: Node) -> Option<Vec<NodeIndex>> {
        match node {
            Node::Recipe(_) => Some(
                self.data
                    .neighbors_directed(self.get_node_idx(node)?, Direction::Outgoing)
                    .collect(),
            ),
            Node::Item(_) => None,
        }
    }

    /// Get all indices of recipes that use the item provided as ingredient.
    /// If the node is not an item or it doesn't exist in graph, None is returned.
    pub fn get_items_as_ingredients_in_recipes_idxs(&self, node: Node) -> Option<Vec<NodeIndex>> {
        match node {
            Node::Item(_) => Some(
                self.data
                    .neighbors_directed(self.get_node_idx(node)?, Direction::Outgoing)
                    .collect(),
            ),
            Node::Recipe(_) => None,
        }
    }

    /// Get all indices of recipes that use the item provided as ingredient.
    /// If the node is not an item or it doesn't exist in graph, None is returned.
    pub fn get_items_as_outputs_in_recipes_idxs(&self, node: Node) -> Option<Vec<NodeIndex>> {
        match node {
            Node::Item(_) => Some(
                self.data
                    .neighbors_directed(self.get_node_idx(node)?, Direction::Incoming)
                    .collect(),
            ),
            Node::Recipe(_) => None,
        }
    }


    pub fn get_node_idx(&self, target_node: Node) -> Option<NodeIndex> {
        self.data
            .node_weights()
            .position(|node| *node == target_node)
            .map(|raw_idx| NodeIndex::from(raw_idx as u32))
    }

    pub fn get_crafting_tree(&self, target: Node) -> Option<Self> {
        todo!()
    }

    pub fn indices_to_nodes(&self, indices: &[NodeIndex]) -> Vec<Node> {
        indices.iter().map(|idx| self.data[*idx]).collect()
    }
}
