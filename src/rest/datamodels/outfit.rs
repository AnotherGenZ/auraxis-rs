use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, SerializeAs, TimestampMilliSeconds, TimestampSeconds};
use crate::{CharacterID, OutfitID};
use crate::realtime::utils::deserialize_from_str;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub outfit_id: OutfitID,
    pub name: String,
    pub name_lower: String,
    pub alias: String,
    pub alias_lower: String,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub time_created: DateTime<Utc>,
    // Do we need this one below since the above field makes it redundant?
    pub time_created_date: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub leader_character_id: CharacterID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub member_count: String,
}
