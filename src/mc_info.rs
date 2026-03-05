use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::mrapi::defines::Version;

pub const LOADERS: &[Loader; 4] = &[
    Loader::Fabric,
    Loader::Quilt,
    Loader::Neoforge,
    Loader::Forge,
];

/// Provide functionality for interacting with minecraft versions
pub trait MCVersionUtils {
    /// check if the version is set to be the latest version.
    fn is_latest(&self) -> bool;
    /// set the version to get the latest version
    fn latest() -> Self;
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub enum VT {
    Release,
    Beta,
    Alpha,
}

impl Display for VT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_write = match self {
            Self::Release => "release",
            Self::Beta => "beta",
            Self::Alpha => "alpha",
        };

        write!(f, "{}", to_write)
    }
}

impl<'de> Deserialize<'de> for VT {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Self::from_str(&String::deserialize(deserializer)?) {
            Ok(vt) => Ok(vt),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

impl FromStr for VT {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "release" => Ok(Self::Release),
            "beta" => Ok(Self::Beta),
            "alpha" => Ok(Self::Alpha),
            _ => Err("invalid version type".to_string()),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
pub enum Loader {
    Fabric,
    Quilt,
    Neoforge,
    Forge,
}

impl<'de> Deserialize<'de> for Loader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Self::from_str(&String::deserialize(deserializer)?) {
            Ok(loader) => Ok(loader),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

impl Display for Loader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_write = match self {
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::Neoforge => "neoforge",
            Self::Forge => "forge",
        };
        write!(f, "{}", to_write)
    }
}

impl FromStr for Loader {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fabric" => Ok(Self::Fabric),
            "neoforge" => Ok(Self::Neoforge),
            "quilt" => Ok(Self::Quilt),
            "forge" => Ok(Self::Forge),
            _ => Err("Unknown Modloader".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MVDescriptor {
    pub mc_ver: MCVersion,
    pub version_types: Vec<VT>,
    pub loader: Loader,
}

impl PartialEq for MVDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.mc_ver == other.mc_ver
            && self.version_types == other.version_types
            && self.loader == other.loader
    }
}

impl MVDescriptor {
    pub fn check_version_compat(&self, version: &Version) -> bool {
        version.game_versions.contains(&self.mc_ver)
            && version.loaders.contains(&self.loader)
            && self.version_types.contains(&version.version_type)
    }
}

pub type MCVersion = String;

impl MCVersionUtils for MCVersion {
    fn is_latest(&self) -> bool {
        self == "latest"
    }

    fn latest() -> Self {
        "latest".to_string()
    }
}
