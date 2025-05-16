use std::str::FromStr;

use serde::{Deserialize, Serialize};
use toml::Table;

use crate::mc_info::{LOADER, VT};

#[derive(Debug, Clone)]
pub enum PackAction {
    CREATE,
    UPDATE,
    MODIFY,
    INSTALL,
}

impl FromStr for PackAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" | "Create" | "CREATE" => Ok(Self::CREATE),
            "update" | "Update" | "UPDATE" => Ok(Self::UPDATE),
            "modify" | "Modify" | "MODIFY" => Ok(Self::MODIFY),
            "install" | "Install" | "INSTALL" => Ok(Self::INSTALL),
            _ => Err("Invalid input".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MVDescriptor {
    pub mc_ver: String,
    pub version_types: Vec<VT>,
    pub loader: LOADER,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pack {
    pub name: String,
    pub version_info: MVDescriptor,
    pub mods: Table,
}

impl Pack {
    pub fn new() -> Self {
        Pack {
            name: "".to_string(),
            version_info: MVDescriptor {
                mc_ver: "".to_string(),
                version_types: vec![VT::RELEASE, VT::BETA, VT::ALPHA],
                loader: LOADER::FABRIC,
            },
            mods: Table::new(),
        }
    }

    pub fn list_mods(&self) {
        println!("The Pack contains the following mods:");
        for (key, info) in &self.mods {
            println!("{key} - {}", info["name"]);
        }
    }
}
