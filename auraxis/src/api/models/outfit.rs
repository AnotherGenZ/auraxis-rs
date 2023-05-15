use crate::api::CensusModel;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Outfit {
    pub outfit_id: Option<String>,
}

impl CensusModel for Outfit {
    fn collection() -> &'static str {
        "outfit"
    }
}
