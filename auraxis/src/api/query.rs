use super::CensusModel;
use crate::api::client::ApiClient;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Query<M: CensusModel> {
    type Output;

    async fn execute(client: &ApiClient) -> Result<Vec<Self::Output>, Box<dyn Error>>;
}
