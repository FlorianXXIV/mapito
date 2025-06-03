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
            Err(e) => Err(serde::de::Error::custom(e)),
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
/// This represents any Given minecraft version.
///
/// if snapshot is false, then the version will be represented as normal ("1.20.1")
/// otherwise it major and minor will be used with ident to represent the
/// version as a snapshot. (22w14a)
///
/// if latest is true, this object will display itself as "latest"
/// the other fields will be ignored.
#[derive(Debug, Clone)]
pub struct MCVersion {
    major: usize,
    minor: usize,
    patch: Option<usize>,
    ident: Option<Vec<char>>,
    latest: bool,
    snapshot: bool,
}

impl Display for MCVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let patch = match self.patch {
            Some(patch) => ".".to_owned() + patch.to_string().as_str(),
            None => "".to_owned(),
        };

        let ident = match &self.ident {
            Some(i) => i.iter().collect(),
            None => "".to_owned(),
        };

        if self.latest {
            write!(f, "latest")
        } else if !self.snapshot {
            write!(f, "{}.{}{}", self.major, self.minor, patch)
        } else {
            write!(f, "{}w{}{}", self.major, self.minor, ident)
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
            return Ok(MCVersion::new());
        }

        let mut is_snap = false;
        let mut patch = None;
        let mut ident: Option<Vec<char>> = None;

        let relreg = Regex::new(r"^([0-9]).([0-9]{1,2})(?:.([0-9]{1,2})){0,1}$").unwrap();
        let snareg = Regex::new(r"^([0-9]{2})w([0-9]{2})([a-z]+)$").unwrap();
        let caps = match relreg.captures(s) {
            Some(caps) => caps,
            None => match snareg.captures(s) {
                Some(caps) => {
                    is_snap = true;
                    caps
                }
                None => return Err("Invalid version Format".to_owned()),
            },
        };

        if !is_snap {
            patch = match caps.get(3) {
                Some(p) => Some(usize::from_str(p.as_str()).expect("patch from str")),
                None => None,
            };
        } else {
            ident = match caps.get(3) {
                Some(i) => Some(i.as_str().chars().collect()),
                None => None,
            }
        }

        Ok(MCVersion {
            major: usize::from_str(caps.get(1).unwrap().as_str()).expect("major from str"),
            minor: usize::from_str(caps.get(2).unwrap().as_str()).expect("minor from str"),
            patch: patch,
            ident: ident,
            latest: false,
            snapshot: is_snap,
        })
    }
}

impl MCVersion {
    /// return a new MCVersion, it will be set to latest
    pub fn new() -> Self {
        MCVersion {
            major: 0,
            minor: 0,
            patch: Some(0),
            ident: None,
            latest: true,
            snapshot: false,
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
        if self.snapshot {
            return false;
        }
        self.patch.is_none()
            && !self.latest
            && self.major == other.major
            && self.minor == other.minor
    }
}

/// MCVersion is equal if x1.y1.z1 == x2.y2.z2 or if both have latest set to true
impl PartialEq for MCVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.snapshot == other.snapshot
            || self.latest == other.latest
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
        if !self.snapshot && other.snapshot || self.snapshot && !other.snapshot{
            return None;
        }
        match self.major.partial_cmp(&other.major) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.minor.partial_cmp(&other.minor) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        if !self.snapshot {
            match self.patch.partial_cmp(&other.patch) {
                ord => return ord,
            }
        } else {
            match self.ident.iter().partial_cmp(&other.ident) {
                ord => return ord,
            }
        }
    }
}
