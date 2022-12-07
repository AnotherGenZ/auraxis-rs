use reqwest::Response;
use serde::{de::Visitor, Deserialize};

use crate::AuraxisError;

#[derive(Debug)]
pub struct CensusResponse {
    pub items: Vec<serde_json::Value>,
    pub count: u32,
}

impl<'de> Deserialize<'de> for CensusResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CensusResponseVisitor)
    }
}

struct CensusResponseVisitor;

impl<'de> Visitor<'de> for CensusResponseVisitor {
    type Value = CensusResponse;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a map with keys '<collection>_list>' and 'returned'"
        )
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut items = None;
        let mut count = None;

        while let Some(k) = map.next_key::<&str>()? {
            if k == "returned" {
                count = Some(map.next_value()?);
            } else if k.ends_with("_list") {
                items = Some(map.next_value()?);
            } else {
                return Err(serde::de::Error::custom(&format!("Invalid key: {}", k)));
            }
        }

        if items.is_none() || count.is_none() {
            return Err(serde::de::Error::custom("Missing collection_list or count"));
        }

        Ok(CensusResponse {
            items: items.unwrap(),
            count: count.unwrap(),
        })
    }
}

impl CensusResponse {
    pub async fn from_response(response: Response) -> Result<Self, AuraxisError> {
        Ok(response.json::<CensusResponse>().await?)
    }
}
