use serde::{Deserialize, Serialize};

use crate::{CharacterID, FactionID};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Model {
    pub character_id: CharacterID,
    pub name: Name,
    pub faction_id: FactionID,
    pub head_id: String,
    pub title_id: String,
    pub times: Times,
    pub certs: Certs,
    pub battle_rank: BattleRank,
    pub profile_id: String,
    pub daily_ribbon: DailyRibbon,
    pub prestige_level: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Name {
    pub first: String,
    pub first_lower: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Times {
    pub creation: String,
    pub creation_date: String,
    pub last_save: String,
    pub last_save_date: String,
    pub last_login: String,
    pub last_login_date: String,
    pub login_count: String,
    pub minutes_played: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Certs {
    pub earned_points: String,
    pub gifted_points: String,
    pub spent_points: String,
    pub available_points: String,
    pub percent_to_next: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BattleRank {
    pub percent_to_next: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DailyRibbon {
    pub count: String,
}
