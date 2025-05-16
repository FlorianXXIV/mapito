use std::{
    fs::{create_dir_all, remove_file, File},
    io::{Read, Write},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{
    config::Configuration,
    mc_info::{LOADER, VT},
};

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
    /// create a new empty pack
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

    /// open the pack file for the given modpack and return Pack object
    pub fn open(name: &String, config: &Configuration) -> Self {
        let mut pack_file = File::open(
            config.pack_path.clone()
                + "/"
                + name
                    .clone()
                    .to_lowercase()
                    .as_str()
                    .replace(" ", "-")
                    .as_str()
                + ".mtpck",
        )
        .expect("open");
        let mut body = String::new();

        pack_file.read_to_string(&mut body).expect("read_to_string");

        let pack = toml::from_str::<Pack>(&body).expect("from_string");

        pack
    }

    /// Print all mods contained in the Pack
    pub fn list_mods(&self) {
        println!("The Pack contains the following mods:");
        for (key, info) in &self.mods {
            println!("{key} - {}", info["name"]);
        }
    }

    /// write this pack to File, at the path given in the config
    pub fn save(&self, config: &Configuration) {
        println!("Saving Changes for {}", self.name);
        create_dir_all(config.pack_path.clone()).expect("create_dir_all");
        let mut pack_fd = File::create(
            config.pack_path.clone()
                + "/"
                + &self.name.to_lowercase().as_str().replace(" ", "-")
                + ".mtpck",
        )
        .expect("create");

        write!(
            &mut pack_fd,
            "{}",
            toml::to_string(self).expect("to_string")
        )
        .expect("write");
    }
    
    /// remove pack from file system
    pub fn remove(&self, config: &Configuration) {
        remove_file(
            config.pack_path.clone()
                + "/"
                + &self.name.to_lowercase().as_str().replace(" ", "-")
                + ".mtpck",
        )
        .expect("remove_file");
    }
}
