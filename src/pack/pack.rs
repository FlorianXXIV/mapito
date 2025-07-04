use std::{
    fmt::Display,
    fs::{create_dir_all, remove_file, File},
    io::{Read, Write},
    str::FromStr,
};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{
    client::Downloader,
    config::Configuration,
    mc_info::{MCVersion, MVDescriptor, LOADER, VT},
    mrapi::client::ApiClient,
    pack::PackMod,
};

#[derive(Debug, Clone)]
pub enum PackAction {
    CREATE,
    UPDATE,
    MODIFY,
    INSTALL,
    REMOVE,
}

impl Display for PackAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_display = match self {
            PackAction::CREATE => "create",
            PackAction::UPDATE => "update",
            PackAction::MODIFY => "modify",
            PackAction::INSTALL => "install",
            PackAction::REMOVE => "remove",
        };
        write!(f, "{}", to_display)
    }
}

impl FromStr for PackAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "create" => Ok(Self::CREATE),
            "update" => Ok(Self::UPDATE),
            "modify" => Ok(Self::MODIFY),
            "install" => Ok(Self::INSTALL),
            "remove" => Ok(Self::REMOVE),
            _ => Err("Invalid input".to_string()),
        }
    }
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
                mc_ver: MCVersion::new(),
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

    /// adds a mod and its dependencies
    pub fn add_mod(&mut self, mod_slug: &String, client: &ApiClient) -> Vec<MCVersion> {
        println!("Looking for {mod_slug}");
        let project_version =
            client.get_project_version(&mod_slug, &self.version_info)
                .expect("get_project_version");
        let mod_version = PackMod {
            name: project_version.name,
            verstion_type: project_version.version_type,
            version_number: project_version.version_number,
            file_url: project_version.files[0].url.clone(),
            sha512: project_version.files[0].hashes["sha512"]
                .to_string()
                .replace("\"", ""),
            file_name: project_version.files[0].filename.clone(),
        };
        self.mods.insert(
            mod_slug.to_string(),
            toml::Value::try_from(&mod_version).expect("try_from"),
        );
        println!(
            "Found mod '{}' and added it to pack",
            mod_version.name.replace("\"", "")
        );
        for dependency in project_version.dependencies {
            let dep_slug = client.get_project( &dependency.project_id)
                .expect("get_project_info")
                .slug;
            if dependency.dependency_type == "required" && !self.mods.contains_key(&dep_slug) {
                println!("Dependency: ");

                self.add_mod(&dep_slug, client);
            }
        }
        project_version.game_versions
    }

    pub fn install(&self, client: &Client, config: &Configuration) {
        for (key, value) in &self.mods {
            let mod_version: PackMod = value.clone().try_into().expect("try_into");
            let dl_path = config.install_path.clone().unwrap() + "/" + &mod_version.file_name;
            println!("Downloading '{key}' to '{dl_path}' ");
            let _ = client.download_file(&dl_path, &mod_version.file_url, &mod_version.sha512);
        }
    }
}

impl Display for Pack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, MC Version: {}, ReleaseTypes: {}, ModLoader: {}",
            self.name,
            self.version_info.mc_ver,
            self.version_info
                .version_types
                .iter()
                .map(|vt| vt.to_string() + " ")
                .collect::<String>(),
            self.version_info.loader.to_string()
        )
    }
}
