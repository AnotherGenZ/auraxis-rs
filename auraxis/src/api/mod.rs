pub mod client;
mod collections;
pub mod models;
mod query;
pub mod request;
mod response;

pub use collections::CensusCollection;
pub use query::Query;
pub use response::CensusResponse;

pub trait CensusModel {
    fn collection() -> &'static str;
}
