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
}

impl IntoFuture for CensusRequest {
    type Output = Result<CensusResponse, AuraxisError>;
    type IntoFuture = impl Future<Output = Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        async move {
            let response = self.client.get(self.url).send().await?;

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

impl From<Join> for String {
    fn from(join: Join) -> Self {
        let show = join.show.map(|show_fields| show_fields.join("'"));

        let hide = join.hide.map(|hide_fields| hide_fields.join("'"));

        let terms = join.terms.map(|terms| {
            terms
                .iter()
                .map(|filter| filter.into())
                .collect::<Vec<String>>()
                .join("'")
        });

        let mut join_formatted = format!(
            "type:{}^on:{}^to:{}^inject_at:{}",
            join.r#type, join.on, join.to, join.inject_at
        );

        if join.list.is_some() && join.list.unwrap() {
            join_formatted += "^list:1";
        }

        match show {
            None => {}
            Some(show) => {
                join_formatted += format!("^show:{}", show).as_str();
            }
        }

        match hide {
            None => {}
            Some(hide) => {
                join_formatted += format!("^hide:{}", hide).as_str();
            }
        }

        match terms {
            None => {}
            Some(terms) => {
                join_formatted += format!("^terms:{}", terms).as_str();
            }
        }

        if let Some(join_type) = join.join_type {
            match join_type {
                JoinType::Inner => join_formatted += "^outer:0",
                JoinType::Outer => join_formatted += "^outer:1",
            }
        }

        join_formatted
    }
}

impl Into<String> for &Join {
    fn into(self) -> String {
        todo!()
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
