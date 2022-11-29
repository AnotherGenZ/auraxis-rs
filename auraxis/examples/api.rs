use auraxis::api::client::{ApiClient, ApiClientConfig};
use auraxis::api::models::Character;
use auraxis::api::{request::FilterType, CensusCollection};
use auraxis::api::{CensusResponse, Query};
use auraxis_macros::Query;
use std::error::Error;

#[derive(Query)]
#[allow(dead_code)]
struct CharacterQuery {
    character: Character,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let client_config = ApiClientConfig::default();

    let client = ApiClient::new(client_config);

    let response = client
        .get(CensusCollection::Character)
        .filter("character_id", FilterType::EqualTo, "5428521211318128657")
        .limit(10)
        .show("name")
        .build()
        .await?;

    println!("{:?}", &response);

    let characters = CharacterQuery::execute(&client).await?;

    println!("{:?}", &characters);

    Ok(())
}
