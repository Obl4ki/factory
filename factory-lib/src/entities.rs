use std::time::Duration;

use rust_decimal::Decimal;

pub type ItemName = String;
pub type RecipeName = String;
pub type ItemAmount = Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Recipe {
    pub name: RecipeName,
    pub results: Vec<(ItemAmount, Item)>,
    pub ingredients: Vec<(ItemAmount, Item)>,
    pub time: Duration,
    pub factory_kind: FactoryKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub name: ItemName,
    pub natural: bool,
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
