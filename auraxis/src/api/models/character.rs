use serde::Deserialize;
use crate::api::CensusModel;

#[derive(Deserialize, Debug)]
pub struct Character {
    pub character_id: Option<String>,
}

impl CensusModel for Character {
    fn collection() -> &'static str {
        "character"
    }
}
