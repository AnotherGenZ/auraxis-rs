use crate::api::CensusModel;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Character {
    pub character_id: Option<String>,
}

impl CensusModel for Character {
    fn collection() -> &'static str {
        "character"
    }
}
