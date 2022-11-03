use std::error::Error;
use tokio;
use auraxis::{client::{RealtimeClient, RealtimeClientConfig}, event::Event};
use auraxis::event::EventNames;
use auraxis::subscription::{CharacterSubscription, EventSubscription, WorldSubscription};
use auraxis::WorldID;
use tracing_subscriber;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();
    let config = RealtimeClientConfig {
        environment: "ps2".to_string(),
        service_id: "example".to_string(),
        realtime_url: None
    };

    let mut client = RealtimeClient::new(config);

    client.subscribe(Some(EventSubscription::All), Some(CharacterSubscription::All), Some(WorldSubscription::Ids(vec![WorldID::Emerald])));

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