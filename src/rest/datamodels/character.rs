use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, SerializeAs, TimestampMilliSeconds, TimestampSeconds};
use crate::realtime::utils::deserialize_from_str;

use crate::{CharacterID, FactionID, HeadID, TitleID};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Model {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    pub name: Name,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub faction_id: FactionID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub head_id: HeadID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub title_id: TitleID,
    pub times: Times,
    pub certs: Certs,
    pub battle_rank: BattleRank,
    pub profile_id: String,
    pub daily_ribbon: DailyRibbon,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub prestige_level: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Name {
    pub first: String,
    pub first_lower: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Times {
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub creation: DateTime<Utc>,
    creation_date: String,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub last_save: DateTime<Utc>,
    last_save_date: String,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub last_login: DateTime<Utc>,
    last_login_date: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub login_count: u64,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub minutes_played: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Certs {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub earned_points: u32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub gifted_points: u32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub spent_points: u32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub available_points: u16,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub percent_to_next: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BattleRank {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub percent_to_next: u8,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub value: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DailyRibbon {
    // TODO: determine correct datatype
    #[serde(deserialize_with = "deserialize_from_str")]
    pub count: u16,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub time: DateTime<Utc>,
    date: String
}
