use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum VT {
    RELEASE,
    BETA,
    ALPHA,
}

impl VT {
    pub fn to_string(&self) -> String {
        match self {
            Self::RELEASE => String::from_str("release").expect("from_str"),
            Self::BETA => String::from_str("beta").expect("from_str"),
            Self::ALPHA => String::from_str("alpha").expect("from_str"),
        }
    }
}
impl<'de> Deserialize<'de> for VT {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "release" | "RELEASE" => Ok(VT::RELEASE),
            "beta" | "BETA" => Ok(VT::BETA),
            "alpha" | "ALPHA" => Ok(VT::ALPHA),
            _ => Err(serde::de::Error::custom("Invalid version Type")),
        }
    }
}

impl FromStr for VT {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "release" | "RELEASE" => Ok(Self::RELEASE),
            "beta" | "BETA" => Ok(Self::BETA),
            "alpha" | "ALPHA" => Ok(Self::ALPHA),
            _ => Err("invalid version type".to_string()),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
pub enum LOADER {
    FABRIC,
    QUILT,
    NEOFORGE,
    FORGE,
}

impl<'de> Deserialize<'de> for LOADER {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "fabric" | "FABRIC" => Ok(LOADER::FABRIC),
            "quilt" | "QUILT" => Ok(LOADER::QUILT),
            "neoforge" | "NEOFORGE" => Ok(LOADER::NEOFORGE),
            "forge" => Ok(LOADER::FORGE),
            _ => Err(serde::de::Error::custom(
                "Expected either fabric, quilt or neoforge",
            )),
        }
    }
}

impl LOADER {
    pub fn to_string(&self) -> String {
        match self {
            Self::FABRIC => "fabric".to_string(),
            Self::QUILT => "quilt".to_string(),
            Self::NEOFORGE => "neoforge".to_string(),
            Self::FORGE => "forge".to_string(),
        }
    }
}

impl FromStr for LOADER {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fabric" | "FABRIC" => Ok(Self::FABRIC),
            "neoforge" | "NEOFORGE" => Ok(Self::NEOFORGE),
            "quilt" | "QUILT" => Ok(Self::QUILT),
            "forge" | "FORGE" => Ok(Self::FORGE),
            _ => Err("Unknown Modloader".to_string()),
        }
    }
}
