use auraxis::realtime::event::EventNames;
use auraxis::realtime::subscription::{
    CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
};
use auraxis::realtime::{
    client::{RealtimeClient, RealtimeClientConfig},
    event::Event,
};
use auraxis::WorldID;
use std::error::Error;

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
        event_names: Some(EventSubscription::Ids(vec![EventNames::PlayerLogin])),
        characters: Some(CharacterSubscription::All),
        worlds: Some(WorldSubscription::Ids(vec![WorldID::Emerald])),
        logical_and_characters_with_worlds: Some(true),
        ..SubscriptionSettings::default()
    };

    let mut client = RealtimeClient::new(config);

    client.subscribe(subscription).await;

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
