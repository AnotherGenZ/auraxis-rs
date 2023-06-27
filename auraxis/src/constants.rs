use std::fmt::Display;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
#[cfg(feature = "strum")]
use strum::{EnumIter, EnumVariantNames, FromRepr};
#[cfg(not(feature = "strum"))]
use std::fmt::Formatter;

#[repr(i16)]
#[derive(
    Serialize, Deserialize, Copy, Clone, Eq, Debug, PartialEq, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(feature = "strum", derive(EnumIter, EnumVariantNames, strum::Display, FromRepr))]
pub enum Loadout {
    Unknown = 0,
    NCInfiltrator = 1,
    NCLightAssault = 3,
    NCMedic = 4,
    NCEngineer = 5,
    NCHeavyAssault = 6,
    NCMAX = 7,
    TRInfiltrator = 8,
    TRLightAssault = 10,
    TRMedic = 11,
    TREngineer = 12,
    TRHeavyAssault = 13,
    TRMAX = 14,
    VSInfiltrator = 15,
    VSLightAssault = 17,
    VSMedic = 18,
    VSEngineer = 19,
    VSHeavyAssault = 20,
    VSMAX = 21,
    NSInfiltrator = 28,
    NSLightAssault = 29,
    NSMedic = 30,
    NSEngineer = 31,
    NSHeavyAssault = 32,
    NSMAX = 45,
}

impl Loadout {
    pub fn get_faction(&self) -> Faction {
        match self {
            Loadout::Unknown => Faction::Unknown,
            Loadout::NCInfiltrator => Faction::NC,
            Loadout::NCLightAssault => Faction::NC,
            Loadout::NCMedic => Faction::NC,
            Loadout::NCEngineer => Faction::NC,
            Loadout::NCHeavyAssault => Faction::NC,
            Loadout::NCMAX => Faction::NC,
            Loadout::TRInfiltrator => Faction::TR,
            Loadout::TRLightAssault => Faction::TR,
            Loadout::TRMedic => Faction::TR,
            Loadout::TREngineer => Faction::TR,
            Loadout::TRHeavyAssault => Faction::TR,
            Loadout::TRMAX => Faction::TR,
            Loadout::VSInfiltrator => Faction::VS,
            Loadout::VSLightAssault => Faction::VS,
            Loadout::VSMedic => Faction::VS,
            Loadout::VSEngineer => Faction::VS,
            Loadout::VSHeavyAssault => Faction::VS,
            Loadout::VSMAX => Faction::VS,
            Loadout::NSInfiltrator => Faction::NS,
            Loadout::NSLightAssault => Faction::NS,
            Loadout::NSMedic => Faction::NS,
            Loadout::NSEngineer => Faction::NS,
            Loadout::NSHeavyAssault => Faction::NS,
            Loadout::NSMAX => Faction::NS,
        }
    }
}

impl FromStr for Loadout {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = i16::from_str(s)?;
        let loadout = Self::try_from(id)?;

        Ok(loadout)
    }
}

#[repr(i16)]
#[derive(
    Serialize, Deserialize, Copy, Clone, Eq, Debug, PartialEq, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(feature = "strum", derive(EnumIter, EnumVariantNames, strum::Display, FromRepr))]
pub enum Faction {
    Unknown = 0,
    VS = 1,
    NC = 2,
    TR = 3,
    NS = 4,
}

impl FromStr for Faction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = i16::from_str(s)?;
        let faction = Self::try_from(id)?;

        Ok(faction)
    }
}

#[repr(i16)]
#[derive(
    Serialize, Deserialize, Copy, Clone, Eq, Debug, PartialEq, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(feature = "strum", derive(EnumIter, EnumVariantNames, strum::Display, FromRepr), strum(ascii_case_insensitive))]
pub enum WorldID {
    Jaeger = 19,
    Briggs = 25,
    Miller = 10,
    Cobalt = 13,
    Connery = 1,
    Emerald = 17,
    Soltech = 40,
}

impl FromStr for WorldID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = i16::from_str(s)?;
        let world = Self::try_from(id)?;

        Ok(world)
    }
}

#[cfg(not(feature = "strum"))]
impl Display for WorldID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldID::Jaeger => write!(f, "Jaeger"),
            WorldID::Briggs => write!(f, "Briggs"),
            WorldID::Miller => write!(f, "Miller"),
            WorldID::Cobalt => write!(f, "Cobalt"),
            WorldID::Connery => write!(f, "Connery"),
            WorldID::Emerald => write!(f, "Emerald"),
            WorldID::Soltech => write!(f, "Soltech"),
        }
    }
}
