use std::{fmt::Display, str::FromStr};

use regex::Regex;

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
            "release" => Ok(Self::RELEASE),
            "beta" => Ok(Self::BETA),
            "alpha" => Ok(Self::ALPHA),
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
        match Self::from_str(&String::deserialize(deserializer)?) {
            Ok(loader) => Ok(loader),
            Err(e) => Err(serde::de::Error::custom(
                e,
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
        match s.to_lowercase().as_str() {
            "fabric" => Ok(Self::FABRIC),
            "neoforge" => Ok(Self::NEOFORGE),
            "quilt" => Ok(Self::QUILT),
            "forge" => Ok(Self::FORGE),
            _ => Err("Unknown Modloader".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MVDescriptor {
    pub mc_ver: MCVersion,
    pub version_types: Vec<VT>,
    pub loader: LOADER,
}
/// This represents any Given minecraft release version
/// patch is Optional as it is not represented every time.
/// if latest is true, this object will display itself as "latest"
/// the other fields will be ignored.
#[derive(Debug, Clone)]
pub struct MCVersion {
    major: usize,
    minor: usize,
    patch: Option<usize>,
    latest: bool,
}

impl Display for MCVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let patch = match self.patch {
            Some(patch) => ".".to_owned() + patch.to_string().as_str(),
            None => "".to_owned(),
        };

        if self.latest {
            write!(f, "latest")
        } else {
            write!(f, "{}.{}{}", self.major, self.minor, patch)
        }
    }
}

/// Serialize a MCVersion into a string like "1.20.1" or "1.20"
impl Serialize for MCVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

/// This deserializes a string like "1.20.1" into a MC Version.
impl<'de> Deserialize<'de> for MCVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Self::from_str(&String::deserialize(deserializer)?) {
            Ok(mc_ver) => Ok(mc_ver),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

impl FromStr for MCVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "latest" {
            return Ok(MCVersion {
                major: 0,
                minor: 0,
                patch: Some(0),
                latest: true,
            });
        }
        let reg = Regex::new(r"^([0-9]).([0-9]{1,2})(?:.([0-9]{1,2})){0,1}$").unwrap();
        let Some(caps) = reg.captures(s) else {
            return Err("Invalid version Format".to_owned());
        };
        Ok(MCVersion {
            major: usize::from_str(caps.get(1).unwrap().as_str()).expect("major from str"),
            minor: usize::from_str(caps.get(2).unwrap().as_str()).expect("minor from str"),
            patch: match caps.get(3) {
                Some(val) => Some(usize::from_str(val.as_str()).expect("patch from str")),
                None => None,
            },
            latest: false,
        })
    }
}

impl MCVersion {
    pub fn new() -> Self {
        MCVersion {
            major: 0,
            minor: 0,
            patch: Some(0),
            latest: true,
        }
    }
    
    /// returns true if other version is considered compatible
    /// versions are considered compatible if they are equal
    /// or if we have no patch version and other has the same major and minor
    /// version.
    pub fn is_compat(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }
        self.patch.is_none() && !self.latest && self.major == other.major && self.minor == other.minor
    }
}

/// MCVersion is equal if x1.y1.z1 == x2.y2.z2 or if both have latest set to true
impl PartialEq for MCVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch || self.latest == other.latest
    }
}

impl PartialOrd for MCVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.latest {
            if other.latest {
                return Some(std::cmp::Ordering::Equal);
            }
            return Some(std::cmp::Ordering::Greater);
        }
        match self.major.partial_cmp(&other.major) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.minor.partial_cmp(&other.minor) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.patch.partial_cmp(&other.patch) {
            Some(core::cmp::Ordering::Equal) => {
                Some(std::cmp::Ordering::Equal)
            }
            ord => return ord,
        }
    }
}
