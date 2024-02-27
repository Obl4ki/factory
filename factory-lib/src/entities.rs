use std::time::Duration;

pub type ItemName = String;
pub type RecipeName = String;
pub type ItemAmount = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Recipe {
    pub name: RecipeName,
    pub result: Vec<(ItemAmount, Item)>,
    pub ingredients: Vec<(ItemAmount, Item)>,
    pub time: Duration,
    pub factory_kind: FactoryKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub name: ItemName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FactoryKind {
    Assembler,
    OilRefinery,
    ChemicalPlant,
    Centrifuge,
    Smelter,
    RocketSilo,
}
