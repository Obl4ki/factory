use crate::{
    entities::{FactoryKind, Item, Recipe},
    error::DataError,
};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fs, path::Path, time::Duration};

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

pub fn load_dataset(json_path: impl AsRef<Path>) -> Result<Vec<Recipe>, DataError> {
    let recipes_str = fs::read_to_string(json_path).map_err(DataError::JsonFileNotFound)?;

    let recipes: HashMap<String, RecipeJson> =
        serde_json::from_str(&recipes_str).map_err(DataError::BadJson)?;

    let data = recipes
        .into_values()
        .map(|rec| Recipe {
            name: rec.name,
            result: rec.products.into_iter().map(|prod| (prod.amount, Item{ name: prod.name })).collect(),
            ingredients: match rec.ingredients {
                IngredientField::Regular(items) => items.into_iter().map(|item| (item.amount, Item{ name: item.name })).collect(),
                IngredientField::Empty {} => vec![],
            },
            time: Duration::from_secs_f64(rec.time),
            factory_kind: match rec.category.as_str() {
                "crafting" | "crafting-with-fluid" | "advanced-crafting" => FactoryKind::Assembler,
                "oil-processing" => FactoryKind::OilRefinery,
                "smelting" => FactoryKind::Smelter,
                "centrifuging" => FactoryKind::Centrifuge,
                "chemistry" => FactoryKind::ChemicalPlant,
                "rocket-building" => FactoryKind::RocketSilo,
                other => {
                    println!("I encountered other kind of crafting recipe: {other}. Defaulting to regular assembler recipe.");
                    FactoryKind::Assembler
                }
            },
        })
        .collect();

    Ok(data)
}