use reqwest::Response;
use serde::{de::IgnoredAny, de::Visitor, Deserialize};

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
                let _ = map.next_value::<IgnoredAny>()?;
            }
        }

        let items =
            items.ok_or_else(|| serde::de::Error::custom("Missing collection_list or count"))?;
        let count =
            count.ok_or_else(|| serde::de::Error::custom("Missing collection_list or count"))?;

        Ok(CensusResponse { items, count })
    }
}

impl CensusResponse {
    pub async fn from_response(response: Response) -> Result<Self, AuraxisError> {
        Ok(response.json::<CensusResponse>().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::CensusResponse;

    #[test]
    fn ignores_extra_metadata_keys() {
        let response = serde_json::from_str::<CensusResponse>(
            r#"{
                "character_list": [{"character_id": "1"}],
                "returned": 1,
                "timing": {"query": "1.23"},
                "errorCode": "ok"
            }"#,
        )
        .expect("response should deserialize");

        assert_eq!(response.count, 1);
        assert_eq!(response.items.len(), 1);
    }
}
