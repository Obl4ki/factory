use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RecipeJsonData {
    id: String,
    name: String,
    r#type: String,
    category: String,
    recipe: Option<Recipe>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Recipe {
    time: Option<f32>,
    r#yield: Option<i32>,
    ingredients: Vec<RecipeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RecipeEntry {
    id: String,
    amount: f32,
}