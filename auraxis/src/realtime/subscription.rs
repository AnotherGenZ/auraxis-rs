use crate::realtime::utils::{
    serialize_all_subscription, serialize_char_ids_subscription, serialize_world_ids_subscription,
};

use crate::realtime::event::EventNames;
use crate::realtime::Service;
use crate::{CharacterID, WorldID};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum CharacterSubscription {
    #[serde(serialize_with = "serialize_all_subscription")]
    All,
    #[serde(serialize_with = "serialize_char_ids_subscription")]
    Ids(Vec<CharacterID>),
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum WorldSubscription {
    #[serde(serialize_with = "serialize_all_subscription")]
    All,
    // TODO: WorldIds enum instead of WorldId u64?
    #[serde(serialize_with = "serialize_world_ids_subscription")]
    Ids(Vec<WorldID>),
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum EventSubscription {
    #[serde(serialize_with = "serialize_all_subscription")]
    All,
    Ids(Vec<EventNames>),
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
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

impl SubscriptionSettings {
    pub fn empty() -> Self {
        Self {
            event_names: None,
            characters: None,
            logical_and_characters_with_worlds: None,
            worlds: None,
            service: Service::Event,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.event_names.is_none()
            && self.characters.is_none()
            && self.worlds.is_none()
            && self.logical_and_characters_with_worlds.is_none()
    }

    pub fn merge(&mut self, other: Self) {
        self.event_names = merge_event_subscription(self.event_names.take(), other.event_names);
        self.characters = merge_character_subscription(self.characters.take(), other.characters);
        self.worlds = merge_world_subscription(self.worlds.take(), other.worlds);
        self.logical_and_characters_with_worlds = other
            .logical_and_characters_with_worlds
            .or(self.logical_and_characters_with_worlds);
        self.service = other.service;
    }

    pub fn clear(&mut self, other: &Self) {
        self.event_names =
            clear_event_subscription(self.event_names.take(), other.event_names.as_ref());
        self.characters =
            clear_character_subscription(self.characters.take(), other.characters.as_ref());
        self.worlds = clear_world_subscription(self.worlds.take(), other.worlds.as_ref());

        if other.logical_and_characters_with_worlds.is_some() {
            self.logical_and_characters_with_worlds = None;
        }
    }
}

fn merge_event_subscription(
    current: Option<EventSubscription>,
    update: Option<EventSubscription>,
) -> Option<EventSubscription> {
    match (current, update) {
        (Some(EventSubscription::All), _) | (_, Some(EventSubscription::All)) => {
            Some(EventSubscription::All)
        }
        (Some(EventSubscription::Ids(mut current)), Some(EventSubscription::Ids(update))) => {
            current.extend(update);
            current.dedup();
            Some(EventSubscription::Ids(current))
        }
        (None, Some(update)) | (Some(update), None) => Some(update),
        (None, None) => None,
    }
}

fn merge_character_subscription(
    current: Option<CharacterSubscription>,
    update: Option<CharacterSubscription>,
) -> Option<CharacterSubscription> {
    match (current, update) {
        (Some(CharacterSubscription::All), _) | (_, Some(CharacterSubscription::All)) => {
            Some(CharacterSubscription::All)
        }
        (
            Some(CharacterSubscription::Ids(mut current)),
            Some(CharacterSubscription::Ids(update)),
        ) => {
            let mut seen = current.iter().copied().collect::<HashSet<_>>();
            for id in update {
                if seen.insert(id) {
                    current.push(id);
                }
            }
            Some(CharacterSubscription::Ids(current))
        }
        (None, Some(update)) | (Some(update), None) => Some(update),
        (None, None) => None,
    }
}

fn merge_world_subscription(
    current: Option<WorldSubscription>,
    update: Option<WorldSubscription>,
) -> Option<WorldSubscription> {
    match (current, update) {
        (Some(WorldSubscription::All), _) | (_, Some(WorldSubscription::All)) => {
            Some(WorldSubscription::All)
        }
        (Some(WorldSubscription::Ids(mut current)), Some(WorldSubscription::Ids(update))) => {
            let mut seen = current.iter().copied().collect::<HashSet<_>>();
            for id in update {
                if seen.insert(id) {
                    current.push(id);
                }
            }
            Some(WorldSubscription::Ids(current))
        }
        (None, Some(update)) | (Some(update), None) => Some(update),
        (None, None) => None,
    }
}

fn clear_event_subscription(
    current: Option<EventSubscription>,
    clear: Option<&EventSubscription>,
) -> Option<EventSubscription> {
    match (current, clear) {
        (None, _) => None,
        (Some(_), Some(EventSubscription::All)) => None,
        (Some(EventSubscription::All), Some(EventSubscription::Ids(_))) => {
            Some(EventSubscription::All)
        }
        (Some(EventSubscription::Ids(current)), Some(EventSubscription::Ids(clear))) => {
            let clear = clear.iter().cloned().collect::<HashSet<_>>();
            let remaining = current
                .into_iter()
                .filter(|event| !clear.contains(event))
                .collect::<Vec<_>>();

            (!remaining.is_empty()).then_some(EventSubscription::Ids(remaining))
        }
        (some, None) => some,
    }
}

fn clear_character_subscription(
    current: Option<CharacterSubscription>,
    clear: Option<&CharacterSubscription>,
) -> Option<CharacterSubscription> {
    match (current, clear) {
        (None, _) => None,
        (Some(_), Some(CharacterSubscription::All)) => None,
        (Some(CharacterSubscription::All), Some(CharacterSubscription::Ids(_))) => {
            Some(CharacterSubscription::All)
        }
        (Some(CharacterSubscription::Ids(current)), Some(CharacterSubscription::Ids(clear))) => {
            let clear = clear.iter().copied().collect::<HashSet<_>>();
            let remaining = current
                .into_iter()
                .filter(|id| !clear.contains(id))
                .collect::<Vec<_>>();

            (!remaining.is_empty()).then_some(CharacterSubscription::Ids(remaining))
        }
        (some, None) => some,
    }
}

fn clear_world_subscription(
    current: Option<WorldSubscription>,
    clear: Option<&WorldSubscription>,
) -> Option<WorldSubscription> {
    match (current, clear) {
        (None, _) => None,
        (Some(_), Some(WorldSubscription::All)) => None,
        (Some(WorldSubscription::All), Some(WorldSubscription::Ids(_))) => {
            Some(WorldSubscription::All)
        }
        (Some(WorldSubscription::Ids(current)), Some(WorldSubscription::Ids(clear))) => {
            let clear = clear.iter().copied().collect::<HashSet<_>>();
            let remaining = current
                .into_iter()
                .filter(|id| !clear.contains(id))
                .collect::<Vec<_>>();

            (!remaining.is_empty()).then_some(WorldSubscription::Ids(remaining))
        }
        (some, None) => some,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
    };
    use crate::realtime::event::EventNames;
    use crate::WorldID;

    #[test]
    fn merge_is_additive() {
        let mut subscription = SubscriptionSettings::empty();
        subscription.merge(SubscriptionSettings {
            event_names: Some(EventSubscription::Ids(vec![EventNames::Death])),
            ..SubscriptionSettings::empty()
        });
        subscription.merge(SubscriptionSettings {
            worlds: Some(WorldSubscription::Ids(vec![WorldID::Emerald])),
            ..SubscriptionSettings::empty()
        });

        assert_eq!(
            subscription.event_names,
            Some(EventSubscription::Ids(vec![EventNames::Death]))
        );
        assert_eq!(
            subscription.worlds,
            Some(WorldSubscription::Ids(vec![WorldID::Emerald]))
        );
    }

    #[test]
    fn clear_removes_requested_entries() {
        let mut subscription = SubscriptionSettings {
            event_names: Some(EventSubscription::Ids(vec![
                EventNames::Death,
                EventNames::PlayerLogin,
            ])),
            characters: Some(CharacterSubscription::Ids(vec![1, 2])),
            worlds: Some(WorldSubscription::Ids(vec![WorldID::Emerald])),
            logical_and_characters_with_worlds: Some(true),
            ..SubscriptionSettings::empty()
        };

        subscription.clear(&SubscriptionSettings {
            event_names: Some(EventSubscription::Ids(vec![EventNames::Death])),
            characters: Some(CharacterSubscription::Ids(vec![2])),
            logical_and_characters_with_worlds: Some(true),
            ..SubscriptionSettings::empty()
        });

        assert_eq!(
            subscription.event_names,
            Some(EventSubscription::Ids(vec![EventNames::PlayerLogin]))
        );
        assert_eq!(
            subscription.characters,
            Some(CharacterSubscription::Ids(vec![1]))
        );
        assert_eq!(subscription.logical_and_characters_with_worlds, None);
    }
}
