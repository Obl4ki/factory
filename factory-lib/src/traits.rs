use std::{fs, path::Path};

use crate::{
    entities::{FactoryKind, Item, Recipe},
    error::{FactoryError, FactoryResult},
};

pub trait DataSource {
    fn from_str(recipes_str: &str, natural_item_names: &[String]) -> FactoryResult<Self>
    where
        Self: std::marker::Sized;

    fn iter_items(&self) -> impl Iterator<Item = &Item>;

    fn iter_recipes(&self) -> impl Iterator<Item = &Recipe>;

    fn from_file(path: impl AsRef<Path>, natural_item_names: &[String]) -> FactoryResult<Self>
    where
        Self: std::marker::Sized,
    {
        let file_content = fs::read_to_string(path).map_err(FactoryError::Io)?;

        Self::from_str(&file_content, natural_item_names)
    }

    fn natural_items(&self) -> Vec<&Item> {
        self.iter_items().filter(|item| item.natural).collect()
    }

    fn try_get_item(&self, name: &str) -> Option<&Item> {
        self.iter_items().find(|item| item.name == name)
    }

    fn get_item(&self, name: &str) -> &Item {
        self.try_get_item(name)
            .unwrap_or_else(|| panic!("Item {name} not found"))
    }

    fn try_get_recipe(&self, name: &str) -> Option<&Recipe> {
        self.iter_recipes().find(|recipe| recipe.name == name)
    }

    fn get_recipe(&self, name: &str) -> &Recipe {
        self.try_get_recipe(name)
            .unwrap_or_else(|| panic!("Recipe {name} not found"))
    }

    fn category_into_factory_kind(category: &str) -> FactoryKind {
        match category {
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
        }
    }
}
