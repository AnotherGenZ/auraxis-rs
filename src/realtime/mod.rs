pub mod client;
pub mod event;
pub mod subscription;
mod utils;

use event::Event;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use subscription::SubscriptionSettings;
use subscription::{CharacterSubscription, EventSubscription, WorldSubscription};
use utils::{deserialize_from_str, serialize_optional_bool};

pub const REALTIME_URL: &str = "wss://push.planetside2.com/streaming";

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Service {
    Event,
}

#[derive(Serialize)]
#[serde(tag = "action", rename_all = "camelCase")]
enum Action {
    Echo {
        payload: serde_json::Value,
        service: Service,
    },
    #[serde(rename_all = "camelCase")]
    Subscribe(SubscriptionSettings),
    #[serde(rename_all = "camelCase")]
    ClearSubscribe {
        #[serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_optional_bool"
        )]
        all: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        event_names: Option<EventSubscription>,
        #[serde(skip_serializing_if = "Option::is_none")]
        characters: Option<CharacterSubscription>,
        #[serde(skip_serializing_if = "Option::is_none")]
        worlds: Option<WorldSubscription>,
        service: Service,
    },
    RecentCharacterIds {
        service: Service,
    },
    RecentCharacterIdsCount {
        service: Service,
    },
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct Subscription {
    pub character_count: u64,
    pub event_names: Vec<String>,
    pub logical_and_characters_with_worlds: bool,
    pub worlds: Vec<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Message {
    ConnectionStateChanged {
        #[serde(deserialize_with = "deserialize_from_str")]
        connected: bool,
    },
    Heartbeat {
        // TODO: EventServerEndpoint / WorldId / request::WorldIds -> bool
        online: HashMap<String, String>,
    },
    ServiceMessage {
        payload: Event,
    },
    ServiceStateChanged {
        #[serde(deserialize_with = "deserialize_from_str")]
        online: bool,
        // TODO: EventServerEndpoint / WorldId / request::WorldIds
        detail: String,
    },
    Subscription {
        subscription: Subscription,
    },
}
