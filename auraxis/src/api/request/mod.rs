mod builder;

pub use builder::CensusRequestBuilder;

use std::future::{Future, IntoFuture};

use reqwest::Client;

use crate::AuraxisError;

use super::response::CensusResponse;

#[derive(Debug, Clone)]
pub struct CensusRequest {
    client: Client,
    collection: String,
    url: String,
    query_params: Vec<(String, String)>,
}

impl IntoFuture for CensusRequest {
    type Output = Result<CensusResponse, AuraxisError>;
    type IntoFuture = impl Future<Output = Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        async move {
            let mut request = self.client.get(self.url);
            if !self.query_params.is_empty() {
                request = request.query(&self.query_params);
            }

            let response = request.send().await?;

            CensusResponse::from_response(response).await
        }
    }
}

#[derive(Debug, Clone)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl From<SortDirection> for &'static str {
    fn from(direction: SortDirection) -> Self {
        match direction {
            SortDirection::Ascending => "1",
            SortDirection::Descending => "-1",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sort {
    field: String,
    direction: SortDirection,
}

#[derive(Debug, Default, Clone)]
pub enum JoinType {
    Inner = 0,
    #[default]
    Outer = 1,
}

#[derive(Debug, Clone)]
pub struct Join {
    r#type: String,
    on: String,
    to: String,
    list: Option<bool>,
    show: Option<Vec<String>>,
    hide: Option<Vec<String>>,
    inject_at: String,
    terms: Option<Vec<Filter>>,
    join_type: Option<JoinType>,
}

impl Join {
    pub fn new(
        r#type: impl Into<String>,
        on: impl Into<String>,
        to: impl Into<String>,
        inject_at: impl Into<String>,
    ) -> Self {
        Self {
            r#type: r#type.into(),
            on: on.into(),
            to: to.into(),
            list: None,
            show: None,
            hide: None,
            inject_at: inject_at.into(),
            terms: None,
            join_type: None,
        }
    }

    pub fn list(mut self, list: bool) -> Self {
        self.list = Some(list);
        self
    }

    pub fn show(mut self, fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.show = Some(fields.into_iter().map(Into::into).collect());
        self
    }

    pub fn hide(mut self, fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.hide = Some(fields.into_iter().map(Into::into).collect());
        self
    }

    pub fn terms(mut self, terms: impl IntoIterator<Item = Filter>) -> Self {
        self.terms = Some(terms.into_iter().collect());
        self
    }

    pub fn join_type(mut self, join_type: JoinType) -> Self {
        self.join_type = Some(join_type);
        self
    }
}

impl From<Join> for String {
    fn from(join: Join) -> Self {
        format_join(&join)
    }
}

impl From<&Join> for String {
    fn from(join: &Join) -> Self {
        format_join(join)
    }
}

#[derive(Debug, Clone)]
pub struct Tree {
    field: String,
    list: Option<bool>,
    prefix: Option<String>,
    start: Option<String>,
}

#[derive(Copy, Clone, Debug)]
pub enum FilterType {
    LessThan,
    LessThanEqualOrEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
    StartsWith,
    Contains,
    EqualTo,
    NotEqualTo,
}

impl From<FilterType> for &'static str {
    fn from(filter_type: FilterType) -> Self {
        match filter_type {
            FilterType::LessThan => "<",
            FilterType::LessThanEqualOrEqualTo => "[",
            FilterType::GreaterThan => ">",
            FilterType::GreaterThanOrEqualTo => "]",
            FilterType::StartsWith => "^",
            FilterType::Contains => "*",
            FilterType::EqualTo => "",
            FilterType::NotEqualTo => "!",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    field: String,
    filter: FilterType,
    value: String,
}

impl From<Filter> for String {
    fn from(filter: Filter) -> Self {
        format!(
            "{}={}{}",
            filter.field,
            <FilterType as Into<&'static str>>::into(filter.filter),
            filter.value
        )
    }
}

impl From<&Filter> for String {
    fn from(filter: &Filter) -> Self {
        format!(
            "{}={}{}",
            filter.field,
            <FilterType as Into<&'static str>>::into(filter.filter),
            filter.value
        )
    }
}

impl Filter {
    pub fn into_pair(self) -> (String, String) {
        (
            self.field,
            format!(
                "{}{}",
                <FilterType as Into<&'static str>>::into(self.filter),
                self.value
            ),
        )
    }
}

fn format_join(join: &Join) -> String {
    let show = join.show.as_ref().map(|show_fields| show_fields.join("'"));

    let hide = join.hide.as_ref().map(|hide_fields| hide_fields.join("'"));

    let terms = join.terms.as_ref().map(|terms| {
        terms
            .iter()
            .map(String::from)
            .collect::<Vec<String>>()
            .join("'")
    });

    let mut join_formatted = format!(
        "type:{}^on:{}^to:{}^inject_at:{}",
        join.r#type, join.on, join.to, join.inject_at
    );

    if join.list == Some(true) {
        join_formatted += "^list:1";
    }

    if let Some(show) = show {
        join_formatted += format!("^show:{}", show).as_str();
    }

    if let Some(hide) = hide {
        join_formatted += format!("^hide:{}", hide).as_str();
    }

    if let Some(terms) = terms {
        join_formatted += format!("^terms:{}", terms).as_str();
    }

    if let Some(join_type) = &join.join_type {
        match join_type {
            JoinType::Inner => join_formatted += "^outer:0",
            JoinType::Outer => join_formatted += "^outer:1",
        }
    }

    join_formatted
}

#[cfg(test)]
mod tests {
    use super::{Filter, FilterType, Join, JoinType};

    #[test]
    fn join_serialization_matches_for_owned_and_borrowed() {
        let join = Join::new("character", "character_id", "character_id", "character")
            .list(true)
            .show(["name.first", "faction_id"])
            .terms([Filter {
                field: "name.first_lower".to_string(),
                filter: FilterType::StartsWith,
                value: "te".to_string(),
            }])
            .join_type(JoinType::Inner);

        let owned = String::from(join.clone());
        let borrowed = String::from(&join);

        assert_eq!(owned, borrowed);
        assert!(owned.contains("^list:1"));
        assert!(owned.contains("^outer:0"));
        assert!(owned.contains("^terms:name.first_lower=^te"));
    }

    #[test]
    fn join_builder_supports_show_only_serialization() {
        let join = Join::new(
            "characters_world",
            "character_id",
            "character_id",
            "characters_world",
        )
        .show(["world_id"]);

        let serialized = String::from(join);

        assert_eq!(
            serialized,
            "type:characters_world^on:character_id^to:character_id^inject_at:characters_world^show:world_id"
        );
    }

    #[test]
    fn filter_into_pair_preserves_operator_in_value() {
        let filter = Filter {
            field: "name.first_lower".to_string(),
            filter: FilterType::StartsWith,
            value: "te st".to_string(),
        };

        assert_eq!(
            filter.into_pair(),
            ("name.first_lower".to_string(), "^te st".to_string())
        );
    }
}
