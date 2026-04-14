use reqwest::Client;

use super::{CensusRequest, Filter, FilterType, Join, Sort, SortDirection, Tree};

pub struct CensusRequestBuilder {
    client: Client,
    collection: String,
    url: String,
    show: Option<Vec<String>>,
    hide: Option<Vec<String>>,
    sort: Option<Vec<Sort>>,
    has: Option<Vec<String>>,
    resolve: Option<Vec<String>>,
    case: Option<bool>,
    limit: Option<u32>,
    limit_per_db: Option<u32>,
    start: Option<u32>,
    include_null: Option<bool>,
    lang: Option<String>,
    join: Option<Vec<Join>>,
    tree: Option<Vec<Tree>>,
    timing: Option<bool>,
    exact_match_first: Option<bool>,
    distinct: Option<String>,
    retry: Option<bool>,
    filters: Option<Vec<Filter>>,
}

impl CensusRequestBuilder {
    pub fn new(client: Client, collection: String, url: String) -> Self {
        Self {
            client,
            collection,
            url,
            show: None,
            hide: None,
            sort: None,
            has: None,
            resolve: None,
            case: None,
            limit: None,
            limit_per_db: None,
            start: None,
            include_null: None,
            lang: None,
            join: None,
            tree: None,
            timing: None,
            exact_match_first: None,
            distinct: None,
            retry: None,
            filters: None,
        }
    }

    pub fn show(mut self, field: impl Into<String>) -> Self {
        match &mut self.show {
            None => {
                self.show = Some(vec![field.into()]);
            }
            Some(show) => {
                show.push(field.into());
            }
        }

        self
    }

    pub fn hide(mut self, field: impl Into<String>) -> Self {
        match &mut self.hide {
            None => {
                self.hide = Some(vec![field.into()]);
            }
            Some(hide) => {
                hide.push(field.into());
            }
        }

        self
    }

    pub fn sort(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        match &mut self.sort {
            None => {
                self.sort = Some(vec![Sort {
                    field: field.into(),
                    direction,
                }]);
            }
            Some(sort) => {
                sort.push(Sort {
                    field: field.into(),
                    direction,
                });
            }
        }

        self
    }

    pub fn has(mut self, field: impl Into<String>) -> Self {
        match &mut self.has {
            None => {
                self.has = Some(vec![field.into()]);
            }
            Some(has) => {
                has.push(field.into());
            }
        }

        self
    }

    pub fn resolve(mut self, field: impl Into<String>) -> Self {
        match &mut self.resolve {
            None => {
                self.resolve = Some(vec![field.into()]);
            }
            Some(resolve) => {
                resolve.push(field.into());
            }
        }

        self
    }

    pub fn case(mut self, should_case: bool) -> Self {
        self.case = Some(should_case);

        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);

        self
    }

    pub fn limit_per_db(mut self, limit_per_db: u32) -> Self {
        self.limit_per_db = Some(limit_per_db);

        self
    }

    pub fn start(mut self, start: u32) -> Self {
        self.start = Some(start);

        self
    }

    pub fn include_null(mut self, include_null: bool) -> Self {
        self.include_null = Some(include_null);

        self
    }

    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = Some(lang.into());

        self
    }

    pub fn join(mut self, join: Join) -> Self {
        match &mut self.join {
            None => {
                self.join = Some(vec![join]);
            }
            Some(joins) => {
                joins.push(join);
            }
        }

        self
    }

    pub fn tree(mut self, tree: Tree) -> Self {
        match &mut self.tree {
            None => {
                self.tree = Some(vec![tree]);
            }
            Some(trees) => {
                trees.push(tree);
            }
        }

        self
    }

    pub fn timing(mut self, value: bool) -> Self {
        self.timing = Some(value);

        self
    }

    pub fn exact_match_first(mut self, value: bool) -> Self {
        self.exact_match_first = Some(value);

        self
    }

    pub fn distinct(mut self, field: impl Into<String>) -> Self {
        self.distinct = Some(field.into());

        self
    }

    pub fn retry(mut self, value: bool) -> Self {
        self.retry = Some(value);

        self
    }

    pub fn filter(
        mut self,
        field: impl Into<String>,
        filter: FilterType,
        value: impl Into<String>,
    ) -> Self {
        let filter = Filter {
            field: field.into(),
            filter,
            value: value.into(),
        };

        match &mut self.filters {
            None => {
                self.filters = Some(vec![filter]);
            }
            Some(filters) => {
                filters.push(filter);
            }
        }

        self
    }

    pub fn build(self) -> CensusRequest {
        let mut query_params = Vec::new();

        match self.filters {
            None => {}
            Some(filters) => {
                for filter in filters {
                    query_params.push(filter.into_pair());
                }
            }
        }

        match self.show {
            None => {}
            Some(show) => {
                let fields = show.join(",");
                query_params.push(("c:show".to_string(), fields));
            }
        }

        match self.hide {
            None => {}
            Some(hide) => {
                let fields = hide.join(",");
                query_params.push(("c:hide".to_string(), fields));
            }
        }

        match self.sort {
            None => {}
            Some(sort) => {
                let fields: String = sort
                    .iter()
                    .map(|field_sort| {
                        format!(
                            "{}:{}",
                            field_sort.field,
                            <SortDirection as Into<&'static str>>::into(
                                field_sort.direction.clone()
                            )
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(",");

                query_params.push(("c:sort".to_string(), fields));
            }
        }

        match self.has {
            None => {}
            Some(has) => {
                query_params.push(("c:has".to_string(), has.join(",")));
            }
        }

        match self.resolve {
            None => {}
            Some(resolve) => {
                query_params.push(("c:resolve".to_string(), resolve.join(",")));
            }
        }

        match self.case {
            None => {}
            Some(case) => {
                query_params.push(("c:case".to_string(), case.to_string()));
            }
        }

        match self.limit {
            None => {}
            Some(limit) => {
                query_params.push(("c:limit".to_string(), limit.to_string()));
            }
        }

        match self.limit_per_db {
            None => {}
            Some(limit_per_db) => {
                query_params.push(("c:limitPerDB".to_string(), limit_per_db.to_string()));
            }
        }

        match self.start {
            None => {}
            Some(start) => {
                query_params.push(("c:start".to_string(), start.to_string()));
            }
        }

        match self.include_null {
            None => {}
            Some(include_null) => {
                query_params.push(("c:includeNull".to_string(), include_null.to_string()));
            }
        }

        match self.lang {
            None => {}
            Some(lang) => {
                query_params.push(("c:lang".to_string(), lang));
            }
        }

        match self.timing {
            None => {}
            Some(timing) => {
                query_params.push(("c:timing".to_string(), timing.to_string()));
            }
        }

        match self.exact_match_first {
            None => {}
            Some(exact_match_first) => {
                query_params.push((
                    "c:exactMatchFirst".to_string(),
                    exact_match_first.to_string(),
                ));
            }
        }

        match self.distinct {
            None => {}
            Some(distinct) => {
                query_params.push(("c:distinct".to_string(), distinct));
            }
        }

        match self.retry {
            None => {}
            Some(retry) => {
                query_params.push(("c:retry".to_string(), retry.to_string()));
            }
        }

        match self.join {
            None => {}
            Some(joins) => {
                let joins: Vec<String> = joins.iter().map(|join| join.into()).collect();

                query_params.push(("c:join".to_string(), joins.join(",")));
            }
        }

        // TODO: Add tree

        CensusRequest {
            client: self.client,
            collection: self.collection,
            url: self.url,
            query_params,
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use super::CensusRequestBuilder;
    use crate::api::request::FilterType;

    #[test]
    fn build_stores_query_params_for_encoded_request_building() {
        let request = CensusRequestBuilder::new(
            Client::new(),
            "character".to_string(),
            "https://example.com/get/ps2:v2/character".to_string(),
        )
        .show("name.first")
        .filter("name.first_lower", FilterType::StartsWith, "te st")
        .build();

        let url = request
            .client
            .get(request.url.clone())
            .query(&request.query_params)
            .build()
            .expect("request should build")
            .url()
            .to_string();

        assert!(url.contains("c%3Ashow=name.first"));
        assert!(url.contains("name.first_lower=%5Ete+st"));
    }
}
