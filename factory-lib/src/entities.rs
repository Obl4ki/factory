#[derive(Debug, Clone)]
pub struct Recipe {
    pub result: Vec<Item>,
    pub ingredients: Vec<Item>,
    pub quantity_per_second: f32,
    pub factory_kind: FactoryKind,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub amount: usize,
}

#[derive(Debug, Clone)]
pub enum FactoryKind {
    Assembler,
    OilRefinery,
    ChemicalPlant,
    Centrifuge,
    Smelter,
}
