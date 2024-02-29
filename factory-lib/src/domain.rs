use std::collections::HashMap;
use std::fmt;

use crate::entities::{Item, ItemAmount, Recipe};

use petgraph::graph::NodeIndex;
use petgraph::prelude::*;

#[derive(Debug, Clone)]
pub struct CraftingGraph<'data> {
    pub data: DiGraph<Node<'data>, ItemAmount>,
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

impl<'a> fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Item(item) => {
                f.write_str(&format!("Item: {}", &item.name))?;
            }
            Node::Recipe(recipe) => {
                f.write_str(&format!("Recipe: {}", &recipe.name))?;
            }
        }

        Ok(())
    }
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
    pub fn new() -> Self {
        Self {
            data: DiGraph::new(),
        }
    }

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

    /// Get all indices of recipes that result in creation of this item.
    /// If the node is not an item or it doesn't exist in graph, None is returned.
    pub fn get_recipes_with_item_in_outputs(&self, node: Node) -> Option<Vec<NodeIndex>> {
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

    /// Starting at the target node, get a list of possible crafting paths an item can have.
    /// Each time an item can be crafted from multiple (N) recipes, this graph will branch into N graphs, which will be processed
    /// further, until each crafting tree is complete.
    /// Passing [`Node::Recipe`] as target will consider the concrete recipe as a starting point, meanwhile
    /// [`Node::Item`] will consider every recipe which result in this item.
    /// If target doesn't exist in graph, then None is returned.

    pub fn get_crafting_trees(&'data self, target: Node<'data>) -> Option<Vec<Self>> {
        let mut complete_subgraphs: Vec<Self> = vec![];

        let target_idx = self.get_node_idx(target)?;

        let mut first_tree: Self = Self::new();
        let subgraph_head_idx = first_tree.data.add_node(target);

        // Subgraph, indexes of nodes in self, indexes of nodes in subgraph
        let mut processing_queue: Vec<(Self, Vec<NodeIndex>, Vec<NodeIndex>)> =
            vec![(first_tree, vec![target_idx], vec![subgraph_head_idx])];

        while let Some((
            mut subgraph,
            mut graph_processing_indices,
            mut subgraph_processing_indices,
        )) = processing_queue.pop()
        {
            if subgraph_processing_indices.is_empty() || graph_processing_indices.is_empty() {
                complete_subgraphs.push(subgraph);
                continue;
            }

            let current_graph_idx = graph_processing_indices.pop()?;
            let current_subgraph_idx = subgraph_processing_indices.pop()?;

            match subgraph.data[current_subgraph_idx] {
                Node::Item(_) => {
                    let recipe_graph_idxs =
                        self.get_recipes_with_item_in_outputs(self.data[current_graph_idx]);

                    if recipe_graph_idxs.as_ref().is_some_and(Vec::is_empty) {
                        processing_queue.push((
                            subgraph,
                            graph_processing_indices,
                            subgraph_processing_indices,
                        ));
                        continue;
                    }

                    for recipe_graph_idx in recipe_graph_idxs? {
                        let recipe = self.data[recipe_graph_idx];

                        let mut branched_subgraph = subgraph.clone();

                        let added_recipe_subgraph_idx = branched_subgraph.data.add_node(recipe);

                        let recipe_output = self
                            .data
                            .edges_connecting(recipe_graph_idx, current_graph_idx)
                            .map(|edge| *edge.weight())
                            .next()?;

                        branched_subgraph.data.add_edge(
                            added_recipe_subgraph_idx,
                            current_subgraph_idx,
                            recipe_output,
                        );

                        let mut branched_graph_processing_indices =
                            graph_processing_indices.clone();

                        branched_graph_processing_indices.push(recipe_graph_idx);

                        let mut branched_subgraph_processing_indices =
                            subgraph_processing_indices.clone();
                        branched_subgraph_processing_indices.push(added_recipe_subgraph_idx);

                        processing_queue.push((
                            branched_subgraph,
                            branched_graph_processing_indices,
                            branched_subgraph_processing_indices,
                        ))
                    }
                }
                Node::Recipe(_) => {
                    let item_graph_idxs = self.get_recipes_input_idxs(self.data[current_graph_idx]);

                    for item_graph_idx in item_graph_idxs? {
                        let item = self.data[item_graph_idx];

                        let added_item_subgraph_idx = subgraph.data.add_node(item);

                        subgraph.data.add_edge(
                            added_item_subgraph_idx,
                            current_subgraph_idx,
                            self.data
                                .edges_connecting(item_graph_idx, current_graph_idx)
                                .map(|edge| *edge.weight())
                                .next()?,
                        );

                        graph_processing_indices.push(item_graph_idx);
                        subgraph_processing_indices.push(added_item_subgraph_idx);
                    }

                    processing_queue.push((
                        subgraph,
                        graph_processing_indices,
                        subgraph_processing_indices,
                    ));
                }
            }
        }

        Some(complete_subgraphs)
    }

    pub fn indices_to_nodes(&self, indices: &[NodeIndex]) -> Vec<Node> {
        indices.iter().map(|idx| self.data[*idx]).collect()
    }
}

impl<'data> Default for CraftingGraph<'data> {
    fn default() -> Self {
        Self::new()
    }
}
