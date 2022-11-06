use serde::{Serialize, Deserialize};

pub mod character;
mod outfit;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Endpoints {
    Character(character::Model)
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Collection {
    pub list: Vec<Endpoints>,
    pub returned: u64,
}