use crate::api::collections::character::CharacterCollection;
use std::fmt::{Display, Formatter};

use async_trait::async_trait;

mod character;

#[async_trait]
pub trait Collection {
    fn name() -> &'static str;
}

#[derive(Debug, Copy, Clone)]
pub enum CensusCollection {
    Character,
}

impl CensusCollection {
    pub fn name(&self) -> &str {
        match self {
            CensusCollection::Character => CharacterCollection::name(),
        }
    }
}

impl From<CensusCollection> for String {
    fn from(val: CensusCollection) -> Self {
        val.name().to_string()
    }
}

impl Display for CensusCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
