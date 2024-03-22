use crate::{
    entities::{Item, Recipe},
    error::{FactoryError, FactoryResult},
    traits,
};
use itertools::Itertools as _;
use rust_decimal::{prelude::FromPrimitive as _, Decimal};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, time::Duration};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RecipeJson {
    name: String,
    ingredients: IngredientField,
    category: String,
    products: Vec<ItemJson>,
    #[serde(rename = "energy")]
    time: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
pub enum IngredientField {
    Regular(Vec<ItemJson>),
    Empty {}, // If recipe doesn't contain any engredients in JSON
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemJson {
    name: String,
    amount: usize,
}

pub struct DataSet {
    pub recipes: Vec<Recipe>,
    pub items: Vec<Item>,
}

impl traits::DataSource for DataSet {
    fn from_str(recipes_str: &str, natural_item_names: &[String]) -> FactoryResult<Self>
    where
        Self: Sized,
    {
        let recipes: HashMap<String, RecipeJson> =
            serde_json::from_str(recipes_str).map_err(FactoryError::JsonMalformed)?;
        let recipes: Vec<Recipe> = recipes
            .into_values()
            .map(|rec| {
                let results: FactoryResult<Vec<(Decimal, Item)>> = rec
                    .products
                    .into_iter()
                    .map(|prod| {
                        Ok((
                            Decimal::from_usize(prod.amount).ok_or_else(|| {
                                FactoryError::CantRepresentAmountAsDecimal(prod.amount)
                            })?,
                            Item {
                                natural: natural_item_names.contains(&prod.name),
                                name: prod.name,
                            },
                        ))
                    })
                    .collect();

                let ingredients: FactoryResult<Vec<(Decimal, Item)>> = match rec.ingredients {
                    IngredientField::Regular(items) => items
                        .into_iter()
                        .map(|item| {
                            Ok((
                                Decimal::from_usize(item.amount).ok_or_else(|| {
                                    FactoryError::CantRepresentAmountAsDecimal(item.amount)
                                })?,
                                Item {
                                    natural: natural_item_names.contains(&item.name),
                                    name: item.name,
                                },
                            ))
                        })
                        .collect(),
                    IngredientField::Empty {} => Ok(vec![]),
                };

                Ok(Recipe {
                    name: rec.name,
                    results: results?,
                    ingredients: ingredients?,
                    time: Duration::from_secs_f64(rec.time),
                    factory_kind: Self::category_into_factory_kind(&rec.category),
                })
            })
            .collect::<FactoryResult<Vec<Recipe>>>()?;
        let items = recipes
            .iter()
            .flat_map(|recipe| recipe.ingredients.iter())
            .chain(recipes.iter().flat_map(|recipe| recipe.results.iter()))
            .map(|(_, item)| item)
            .unique()
            .cloned()
            .collect();

        Ok(Self { recipes, items })
    }

    fn iter_items(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }

    fn iter_recipes(&self) -> impl Iterator<Item = &Recipe> {
        self.recipes.iter()
    }
}

impl DataSet {
    pub fn natural_items(&self) -> Vec<&Item> {
        self.items.iter().filter(|item| item.natural).collect()
    }

    pub fn try_get_item(&self, name: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.name == name)
    }

    pub fn get_item(&self, name: &str) -> &Item {
        self.items
            .iter()
            .find(|item| item.name == name)
            .unwrap_or_else(|| panic!("Item {name} not found"))
    }

    pub fn try_get_recipe(&self, name: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|recipe| recipe.name == name)
    }

    pub fn get_recipe(&self, name: &str) -> &Recipe {
        self.recipes
            .iter()
            .find(|recipe| recipe.name == name)
            .unwrap_or_else(|| panic!("Recipe {name} not found"))
    }

    /// Sorts the database item's and recipe's alphabetically by names.
    pub fn sorted_by_names(mut self) -> Self {
        self.items
            .sort_by(|item1, item2| item1.name.cmp(&item2.name));
        self.recipes
            .sort_by(|recipe1, recipe2| recipe1.name.cmp(&recipe2.name));

        self
    }
}
