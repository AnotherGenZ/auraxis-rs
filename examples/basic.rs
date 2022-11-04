use std::error::Error;
use tokio;
use auraxis::{client::{RealtimeClient, RealtimeClientConfig}, event::Event};
use auraxis::event::EventNames;
use auraxis::subscription::{SubscriptionSettings, EventSubscription, CharacterSubscription, WorldSubscription};
use auraxis::WorldID;
use tracing_subscriber;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();
    let config = RealtimeClientConfig {
        service_id: "example".to_string(),
        ..RealtimeClientConfig::default()
    };

    let subscription = SubscriptionSettings {
        event_names: Some(EventSubscription::Ids(vec!(EventNames::PlayerLogin))),
        characters: Some(CharacterSubscription::All),
        worlds: Some(WorldSubscription::Ids(vec![WorldID::Emerald])),
        logical_and_characters_with_worlds: Some(true),
        ..SubscriptionSettings::default()
    };

    let mut client = RealtimeClient::new(config);

    client.subscribe(subscription);

    let mut events = client.connect().await?;

    while let Some(event) = events.recv().await {
        match &event {
            Event::PlayerLogin(player) => {
                println!("Player {} logged in", player.character_id);
            }
            _ => {
                println!("{:?}", &event);
            }
        }
    }

    Ok(())
}