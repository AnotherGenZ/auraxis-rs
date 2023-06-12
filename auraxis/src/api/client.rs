use crate::api::collections::CensusCollection;
use crate::api::request::CensusRequestBuilder;

use reqwest::Client;

pub const API_URL: &str = "https://census.daybreakgames.com";

#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    pub environment: Option<String>,
    pub service_id: Option<String>,
    pub api_url: Option<String>,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            environment: Some(String::from("ps2:v2")),
            service_id: None,
            api_url: Some(String::from(API_URL)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    environment: String,
    base_url: String,
    http_client: Client,
}

impl ApiClient {
    pub fn new(config: ApiClientConfig) -> Self {
        let base_url = match config.api_url {
            None => String::from(API_URL),
            Some(url) => url,
        };

        let base_url = match config.service_id {
            None => base_url,
            Some(service_id) => base_url + &*format!("/s:{}", service_id),
        };

        let environment = match config.environment {
            None => "ps2:v2".into(),
            Some(env) => env,
        };

        Self {
            base_url,
            environment,
            http_client: Client::new(),
        }
    }

    pub fn get(&self, collection: impl Into<String> + Clone) -> CensusRequestBuilder {
        let url = format!("{}/get/{}", &self.base_url, &self.environment);

        let url = format!("{}/{}", url, collection.clone().into());

        CensusRequestBuilder::new(self.http_client.clone(), collection.into(), url)
    }

    pub async fn count(&self, _collection: CensusCollection) {
        let _url = format!("{}/count/{}", &self.base_url, &self.environment);
    }
}
