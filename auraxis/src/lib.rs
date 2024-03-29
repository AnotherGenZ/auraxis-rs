#![cfg_attr(
    feature = "api",
    feature(impl_trait_in_assoc_type)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

#[cfg(feature = "api")]
pub mod api;
mod constants;
pub mod realtime;

pub use constants::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuraxisError {
    #[error("Websocket error")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Ser(de) error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Http error")]
    #[cfg(feature = "api")]
    HttpError(#[from] reqwest::Error),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

pub type CharacterID = u64;
pub type OutfitID = u64;
pub type ZoneID = u32;
pub type FacilityID = u32;
pub type ExperienceID = u16;
pub type VehicleID = u16;
pub type WeaponID = u32;
pub type FiremodeID = u32;
