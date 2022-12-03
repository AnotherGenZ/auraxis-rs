use crate::realtime::utils::{
    serialize_all_subscription, serialize_char_ids_subscription, serialize_world_ids_subscription,
};

use crate::realtime::event::EventNames;
use crate::realtime::Service;
use crate::{CharacterID, WorldID};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum CharacterSubscription {
    #[serde(serialize_with = "serialize_all_subscription")]
    All,
    #[serde(serialize_with = "serialize_char_ids_subscription")]
    Ids(Vec<CharacterID>),
}

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum WorldSubscription {
    #[serde(serialize_with = "serialize_all_subscription")]
    All,
    // TODO: WorldIds enum instead of WorldId u64?
    #[serde(serialize_with = "serialize_world_ids_subscription")]
    Ids(Vec<WorldID>),
}

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum EventSubscription {
    #[serde(serialize_with = "serialize_all_subscription")]
    All,
    Ids(Vec<EventNames>),
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_names: Option<EventSubscription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characters: Option<CharacterSubscription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_and_characters_with_worlds: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worlds: Option<WorldSubscription>,
    pub service: Service,
}

impl Default for SubscriptionSettings {
    fn default() -> Self {
        Self {
            event_names: Some(EventSubscription::All),
            characters: Some(CharacterSubscription::All),
            logical_and_characters_with_worlds: None,
            worlds: Some(WorldSubscription::All),
            service: Service::Event,
        }
    }
}
