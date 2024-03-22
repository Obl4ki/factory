use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::io::Write as _;

use std::path::Path;
use std::process::{Command, Stdio};
use std::{cmp, fmt, fs};

use crate::entities::{Item, ItemAmount, Recipe};
use crate::error::FactoryResult;
use crate::prelude::FactoryError;
use crate::traits::DataSource;

use itertools::Itertools;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::prelude::*;

#[derive(Debug, Clone)]
pub struct CraftingGraph<'data> {
    pub data: DiGraph<Node<'data>, ItemAmount>,
    natural_items: Vec<&'data Item>,
}

impl cmp::PartialEq for CraftingGraph<'_> {
    fn eq(&self, other: &Self) -> bool {
        let self_tiers = self.iter_nodes().map(|node| node.get_tier()).sum::<Tier>();
        let other_tiers = other.iter_nodes().map(|node| node.get_tier()).sum::<Tier>();
        self_tiers == other_tiers
    }
}

impl cmp::Eq for CraftingGraph<'_> {}

impl<'data, D> From<&'data D> for CraftingGraph<'data>
where
    D: DataSource,
{
    fn from(data: &'data D) -> Self {
        CraftingGraph {
            data: DiGraph::new(),
            natural_items: data.natural_items(),
        }
    }
}

type Tier = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node<'data> {
    Item(&'data Item, Tier),
    Recipe(&'data Recipe, Tier),
}

impl Node<'_> {
    fn get_tier(&self) -> Tier {
        match self {
            Node::Item(_, tier) | Node::Recipe(_, tier) => *tier,
        }
    }

    fn set_tier(&mut self, new_tier: Tier) {
        match self {
            Node::Item(_, tier) | Node::Recipe(_, tier) => *tier = new_tier,
        }
    }
}

impl fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Item(item, tier) => {
                f.write_str(&format!("{} [{}]", item.name, tier))?;
            }
            Node::Recipe(recipe, tier) => {
                f.write_str(&format!("{} | {:?} [{}]", &recipe.name, &recipe.time, tier))?;
            }
        }

        Ok(())
    }
}

impl cmp::PartialOrd for CraftingGraph<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for CraftingGraph<'_> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let self_score = self.iter_nodes().map(|node| node.get_tier()).sum::<Tier>();

        let other_score = other.iter_nodes().map(|node| node.get_tier()).sum::<Tier>();

        other_score
            .partial_cmp(&self_score)
            .unwrap_or(Ordering::Equal)
    }
}

impl<'data> CraftingGraph<'data> {
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
    pub fn iter_nodes(&self) -> impl Iterator<Item = Node> {
        self.data.node_weights().copied()
    }

    pub fn get_item_node(&self, item_name: &str) -> Node {
        self.iter_nodes()
            .find(|item| matches!(item, Node::Item(Item {name, .. }, _) if name == item_name))
            .unwrap_or_else(|| panic!("Recipe {item_name} not found"))
    }

    pub fn get_recipe_node(&self, recipe_name: &str) -> Node {
        self.iter_nodes()
            .find(|recipe| matches!(recipe, Node::Recipe(Recipe {name, .. }, _) if name == recipe_name))
            .unwrap_or_else(|| panic!("Recipe {recipe_name} not found"))
    }

    pub fn from_dataset<D: DataSource>(dataset: &'data D) -> Self {
        let mut graph = Self::from(dataset);

        let mut current_indices: Vec<NodeIndex> = vec![];
        let mut visited = HashSet::new();

        for natural in &graph.natural_items {
            let idx = graph.data.add_node(Node::Item(natural, 0));
            current_indices.push(idx);
        }

        while let Some(current_idx) = current_indices.pop() {
            if visited.contains(&current_idx) {
                continue;
            }

            match graph.data[current_idx] {
                Node::Item(item, tier) => {
                    let recipes_depending_on_item = dataset.iter_recipes().filter(|rec| {
                        rec.ingredients
                            .iter()
                            .any(|(_, ingredient)| ingredient.name == item.name)
                    });

                    for recipe in recipes_depending_on_item {
                        let mut maybe_recipe_idx = graph.get_recipe_idx_from_name(&recipe.name);

                        let recipe_idx = maybe_recipe_idx.get_or_insert_with(|| {
                            graph.data.add_node(Node::Recipe(recipe, tier + 1))
                        });

                        let input_amount =
                            recipe
                                .ingredients
                                .iter()
                                .find_map(|(amount, ingredient_item)| {
                                    if item.name == ingredient_item.name {
                                        Some(*amount)
                                    } else {
                                        None
                                    }
                                });

                        if let Some(weight) = input_amount {
                            graph.data.update_edge(current_idx, *recipe_idx, weight);
                        }

                        current_indices.push(*recipe_idx);
                    }
                }

                Node::Recipe(recipe, tier) => {
                    for (amount, item) in &recipe.results {
                        let mut maybe_item_idx = graph.get_item_idx_from_name(&item.name);

                        let item_idx = maybe_item_idx
                            .get_or_insert_with(|| graph.data.add_node(Node::Item(item, tier + 1)));

                        graph.data.add_edge(current_idx, *item_idx, *amount);
                        current_indices.push(*item_idx);
                    }
                }
            }

            visited.insert(current_idx);
        }

        graph.adjust_tiers();

        graph
    }

    pub fn adjust_tiers(&mut self) {
        let mut current_indices: VecDeque<NodeIndex> = VecDeque::new();
        let mut visited = HashSet::new();

        for natural in &self.natural_items {
            let idx = self
                .get_node_idx(Node::Item(natural, 0))
                .unwrap_or_else(|| self.data.add_node(Node::Item(natural, 0)));

            current_indices.push_back(idx);
        }

        while let Some(current_idx) = current_indices.pop_front() {
            if visited.contains(&current_idx) {
                continue;
            }

            match self.data[current_idx] {
                Node::Item(item, _) => {
                    let recipes_tier_min = self
                        .get_recipes_with_item_in_outputs(self.data[current_idx])
                        .unwrap_or_default()
                        .iter()
                        .map(|recipes_idx| self.data[*recipes_idx].get_tier())
                        .min()
                        .unwrap_or(1);

                    self.data[current_idx].set_tier(recipes_tier_min + 1);

                    if item.natural {
                        self.data[current_idx].set_tier(1);
                    }

                    current_indices.extend(
                        self.get_items_as_ingredients_in_recipes_idxs(self.data[current_idx])
                            .unwrap_or_default(),
                    );
                }
                Node::Recipe(_, _) => {
                    let input_items = self.get_ingredients_for_recipe_idx(self.data[current_idx]);

                    match input_items {
                        // Defer computation of this node until every ingredient has its tier computed
                        Some(ingredients)
                            if ingredients.iter().any(|idx| !visited.contains(idx)) =>
                        {
                            current_indices.push_back(current_idx);
                            continue;
                        }

                        Some(ingredients) => {
                            let ingredients_tier_sum = ingredients
                                .into_iter()
                                .map(|idx| self.data[idx])
                                .map(|node| node.get_tier())
                                .sum::<Tier>();

                            self.data[current_idx].set_tier(ingredients_tier_sum + 1);

                            current_indices.extend(
                                self.get_results_for_recipe_idxs(self.data[current_idx])
                                    .unwrap_or_default(),
                            );
                        }
                        None => {
                            self.data[current_idx].set_tier(0);
                        }
                    }
                }
            }
            visited.insert(current_idx);
        }
    }

    /// Get all indices of item nodes that are direct input items to the recipe provided.
    /// If the node is not a recipe or it doesn't exist in graph, None is returned.
    pub fn get_ingredients_for_recipe_idx(&self, node: Node) -> Option<Vec<NodeIndex>> {
        match node {
            Node::Recipe(..) => Some(
                self.data
                    .neighbors_directed(self.get_node_idx(node)?, Direction::Incoming)
                    .sorted_by(|idx1, idx2| {
                        let tier1 = self.data[*idx1].get_tier();
                        let tier2 = self.data[*idx2].get_tier();
                        tier1.cmp(&tier2)
                    })
                    .collect(),
            ),
            Node::Item(..) => None,
        }
    }

    /// Get all indices of item nodes that are direct output items to the recipe provided.
    /// If the node is not a recipe or it doesn't exist in graph, None is returned.
    pub fn get_results_for_recipe_idxs(&self, node: Node) -> Option<Vec<NodeIndex>> {
        match node {
            Node::Recipe(..) => Some(
                self.data
                    .neighbors_directed(self.get_node_idx(node)?, Direction::Outgoing)
                    .sorted_by(|idx1, idx2| {
                        let tier1 = self.data[*idx1].get_tier();
                        let tier2 = self.data[*idx2].get_tier();
                        tier1.cmp(&tier2)
                    })
                    .collect(),
            ),
            Node::Item(..) => None,
        }
    }

    /// Get all indices of recipes that use the item provided as ingredient.
    /// If the node is not an item or it doesn't exist in graph, None is returned.
    pub fn get_items_as_ingredients_in_recipes_idxs(&self, node: Node) -> Option<Vec<NodeIndex>> {
        match node {
            Node::Item(..) => Some(
                self.data
                    .neighbors_directed(self.get_node_idx(node)?, Direction::Outgoing)
                    .sorted_by(|idx1, idx2| {
                        let tier1 = self.data[*idx1].get_tier();
                        let tier2 = self.data[*idx2].get_tier();
                        tier1.cmp(&tier2)
                    })
                    .collect(),
            ),
            Node::Recipe(..) => None,
        }
    }

    /// Get all indices of recipes that result in creation of this item.
    /// If the node is not an item or it doesn't exist in graph, None is returned.
    pub fn get_recipes_with_item_in_outputs(&self, node: Node) -> Option<Vec<NodeIndex>> {
        match node {
            Node::Item(..) => Some(
                self.data
                    .neighbors_directed(self.get_node_idx(node)?, Direction::Incoming)
                    .sorted_by(|idx1, idx2| {
                        let tier1 = self.data[*idx1].get_tier();
                        let tier2 = self.data[*idx2].get_tier();
                        tier1.cmp(&tier2)
                    })
                    .collect(),
            ),
            Node::Recipe(..) => None,
        }
    }

    pub fn get_node_idx(&self, target_node: Node) -> Option<NodeIndex> {
        self.data
            .node_weights()
            .position(|node| *node == target_node)
            .map(|raw_idx| NodeIndex::from(raw_idx as u32))
    }

    pub fn get_item_idx_from_name(&self, item_name: &str) -> Option<NodeIndex> {
        self.data
            .node_weights()
            .position(|node| match node {
                Node::Item(Item { name, .. }, _) => item_name == name,
                _ => false,
            })
            .map(|raw_idx| NodeIndex::from(raw_idx as u32))
    }

    pub fn get_recipe_idx_from_name(&self, recipe_name: &str) -> Option<NodeIndex> {
        self.data
            .node_weights()
            .position(|node| match node {
                Node::Recipe(Recipe { name, .. }, _) => recipe_name == name,
                _ => false,
            })
            .map(|raw_idx| NodeIndex::from(raw_idx as u32))
    }

    /// Starting at the target node, get a list of possible crafting paths an item can have.
    /// Each time an item can be crafted from multiple (N) recipes, this graph will branch into N graphs, which will be processed
    /// further, until each crafting tree is complete.
    /// Passing [`Node::Recipe`] as target will consider the concrete recipe as a starting point, meanwhile
    /// [`Node::Item`] will consider every recipe which result in this item.
    /// If target doesn't exist in graph, then None is returned.
    pub fn get_crafting_trees(
        &'data self,
        target: Node<'data>,
        max_number_of_solutions: usize,
    ) -> Option<Vec<Self>> {
        let mut complete_subgraphs: Vec<Self> = vec![];

        let target_idx = self.get_node_idx(target)?;

        let mut first_tree = Self {
            data: DiGraph::new(),
            natural_items: self.natural_items.clone(),
        };
        let subgraph_head_idx = first_tree.data.add_node(target);

        let mut processing_queue: BinaryHeap<(Self, Vec<(NodeIndex, NodeIndex)>)> =
            BinaryHeap::from([(first_tree, vec![(target_idx, subgraph_head_idx)])]);

        while let Some((mut subgraph, mut processing_indices)) = processing_queue.pop() {
            if processing_indices.is_empty() {
                println!("Found possibility with len {}", subgraph.data.node_count());
                complete_subgraphs.push(subgraph);
                continue;
            }

            if complete_subgraphs.len() >= max_number_of_solutions {
                break;
            }

            let (current_graph_idx, current_subgraph_idx) = processing_indices.pop()?;

            match subgraph.data[current_subgraph_idx] {
                Node::Item(item, _) => {
                    let recipe_graph_idxs = self
                        .get_recipes_with_item_in_outputs(self.data[current_graph_idx])
                        .map(|mut recipe_idxs| {
                            recipe_idxs.sort_by(|&idx1, &idx2| {
                                self.data[idx1].get_tier().cmp(&self.data[idx2].get_tier())
                            });
                            recipe_idxs
                        });

                    if item.natural {
                        processing_queue.push((subgraph, processing_indices));
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

                        let mut branched_processing_indices = processing_indices.clone();

                        branched_processing_indices
                            .push((recipe_graph_idx, added_recipe_subgraph_idx));

                        processing_queue.push((branched_subgraph, branched_processing_indices))
                    }
                }
                Node::Recipe(_, _) => {
                    let item_graph_idxs =
                        self.get_ingredients_for_recipe_idx(self.data[current_graph_idx]);

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

                        if subgraph.copy_of_node_is_present_in_ancestors(item, current_subgraph_idx)
                        {
                            continue;
                        }

                        processing_indices.push((item_graph_idx, added_item_subgraph_idx));
                    }

                    processing_queue.push((subgraph, processing_indices));
                }
            }
        }

        Some(complete_subgraphs)
    }

    //
    #[allow(unused)]
    pub fn with_input_constraints<C>(&self, input_constraints: C) -> Self
    where
        C: IntoIterator<Item = (&'data Item, f32)>,
    {
        let input_c: HashMap<&Item, f32> = input_constraints.into_iter().collect();

        self.clone()
    }

    pub fn indices_to_nodes(&self, indices: &[NodeIndex]) -> Vec<Node> {
        indices.iter().map(|idx| self.data[*idx]).collect()
    }

    fn copy_of_node_is_present_in_ancestors(
        &self,
        node: Node,
        parent_of_node_idx: NodeIndex,
    ) -> bool {
        let mut dfs = Dfs::new(&self.data, parent_of_node_idx);
        while let Some(idx) = dfs.next(&self.data) {
            if self.data[idx] == node {
                return true;
            }
        }

        false
    }

    pub fn to_dot(&self) -> String {
        // Config::_Incomplete gives the best drawing despite being WIP
        format!(
            "{}",
            Dot::with_config(&self.data, &[Config::_Incomplete(())])
        )
    }

    pub fn save_as_svg(&self, file_name: impl AsRef<Path>) -> FactoryResult<()> {
        let dot = self.to_dot();
        let mut cmd = Command::new("dot")
            .arg("-Tsvg")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        {
            let mut stdin = cmd.stdin.take().ok_or(FactoryError::CommandSpawn(
                "Failed to take stdin".to_string(),
            ))?;

            stdin.write_all(dot.as_bytes())?;
        }

        let output = cmd.wait_with_output()?;

        if !output.stderr.is_empty() {
            println!("stderr: {}", std::str::from_utf8(&output.stderr)?);
        }

        let mut file = fs::File::create(file_name)?;
        file.write_all(&output.stdout)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use itertools::Itertools;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::{
        entities::{FactoryKind, Item, Recipe},
        traits::{self, DataSource},
    };

    use super::{CraftingGraph, Node, Tier};

    struct DataSetMock {
        items: Vec<Item>,
        recipes: Vec<Recipe>,
    }

    impl traits::DataSource for DataSetMock {
        fn from_str(
            _recipes_str: &str,
            _natural_item_names: &[String],
        ) -> crate::error::FactoryResult<Self>
        where
            Self: std::marker::Sized,
        {
            Ok(Self::new())
        }

        fn iter_items(&self) -> impl Iterator<Item = &Item> {
            self.items.iter()
        }

        fn iter_recipes(&self) -> impl Iterator<Item = &Recipe> {
            self.recipes.iter()
        }
    }

    impl DataSetMock {
        fn new() -> Self {
            let natural_items = ["iron-ore", "copper-ore"].into_iter().map(|name| Item {
                name: name.to_string(),
                natural: true,
            });

            let other_items = [
                "iron-plate",
                "copper-plate",
                "copper-cable",
                "electronic-circuit",
            ]
            .into_iter()
            .map(|name| Item {
                name: name.to_string(),
                natural: false,
            });

            let items = natural_items.chain(other_items).collect_vec();

            let item = |name: &str| -> Item {
                items
                    .iter()
                    .find(|item| item.name == name)
                    .unwrap_or_else(|| panic!("Item not found: {name}"))
                    .clone()
            };

            let recipe = |name: &str,
                          time: f64,
                          inputs: &[(Decimal, Item)],
                          outputs: &[(Decimal, Item)],
                          kind: FactoryKind| {
                Recipe {
                    name: name.to_string(),
                    results: outputs.to_vec(),
                    ingredients: inputs.to_vec(),
                    time: Duration::from_secs_f64(time),
                    factory_kind: kind,
                }
            };

            let recipes = vec![
                recipe(
                    "copper-plate",
                    3.2,
                    &[(dec!(1), item("copper-ore"))],
                    &[(dec!(1), item("copper-plate"))],
                    FactoryKind::Assembler,
                ),
                recipe(
                    "copper-cable",
                    0.5,
                    &[(dec!(1), item("copper-plate"))],
                    &[(dec!(2), item("copper-cable"))],
                    FactoryKind::Assembler,
                ),
                recipe(
                    "iron-plate",
                    3.2,
                    &[(dec!(1), item("iron-ore"))],
                    &[(dec!(1), item("iron-plate"))],
                    FactoryKind::Assembler,
                ),
                recipe(
                    "electronic-circuit",
                    0.5,
                    &[
                        (dec!(3), item("copper-cable")),
                        (dec!(1), item("iron-plate")),
                    ],
                    &[(dec!(1), item("electronic-circuit"))],
                    FactoryKind::Assembler,
                ),
            ];

            Self { recipes, items }
        }
    }

    #[test]
    fn test_crafting_graph_structure() {
        let data = DataSetMock::new();
        let graph = CraftingGraph::from_dataset(&data);
        assert_eq!(
            graph.iter_nodes().count(),
            data.recipes.len() + data.items.len()
        );

        for item in data.iter_items() {
            assert_eq!(data.try_get_item(&item.name), Some(item));
        }

        for recipe in data.iter_recipes() {
            assert_eq!(data.try_get_recipe(&recipe.name), Some(recipe));
        }
    }

    #[test]
    fn test_tiers() {
        let data = DataSetMock::new();
        let graph = CraftingGraph::from_dataset(&data);
        let expected_item_tiers: Vec<(&Item, Tier)> = vec![
            (data.get_item("copper-ore"), 0),
            (data.get_item("copper-plate"), 2),
            (data.get_item("iron-ore"), 0),
            (data.get_item("iron-plate"), 2),
            (data.get_item("electronic-circuit"), 6),
        ];
        let expected_recipe_tiers: Vec<(&Recipe, Tier)> = vec![
            (data.get_recipe("copper-plate"), 1),
            (data.get_recipe("iron-plate"), 1),
            (data.get_recipe("electronic-circuit"), 5),
        ];

        let nodes: Vec<Node> = graph.data.node_weights().copied().collect();
        dbg!(&nodes);

        for (item, tier) in expected_item_tiers {
            let found = nodes.contains(&Node::Item(item, tier));

            assert!(
                found,
                "Item {} with tier {} was not present in nodes",
                &item.name, &tier
            );
        }

        for (recipe, tier) in expected_recipe_tiers {
            let found = nodes.contains(&Node::Recipe(recipe, tier));

            assert!(
                found,
                "Item {} with tier {} was not present in nodes",
                &recipe.name, &tier
            );
        }
    }
}
