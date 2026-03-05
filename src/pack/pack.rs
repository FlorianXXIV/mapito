use core::panic;
use std::{
    fmt::Display,
    fs::{create_dir_all, read_dir, remove_file, File},
    io::{Read, Write},
};

use clap::Subcommand;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{
    cli::input::confirm_input,
    client::Downloader,
    config::Configuration,
    mc_info::{Loader, MCVersion, MVDescriptor, VT},
    mrapi::client::ApiClient,
    pack::PackMod,
};

#[derive(Debug, Clone, Subcommand)]
pub enum PackAction {
    /// Create a new pack
    Create,
    /// Update an existing pack
    Update,
    /// Modify an existing pack
    Modify,
    /// Install an existing pack
    Install,
    /// Remove an existing pack
    Remove,
    /// List all packs
    List,
}

impl Display for PackAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_display = match self {
            PackAction::Create => "create",
            PackAction::Update => "update",
            PackAction::Modify => "modify",
            PackAction::Install => "install",
            PackAction::Remove => "remove",
            PackAction::List => "list",
        };
        write!(f, "{}", to_display)
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
                version_types: vec![VT::Release, VT::Beta, VT::Alpha],
                loader: Loader::Fabric,
            },
            mods: Table::new(),
        }
    }

    /// open the pack file for the given modpack and return Pack object
    pub fn open(name: &str, config: &Configuration) -> Self {
        let mut pack_file = File::open(
            config.pack_path.clone()
                + "/"
                + name.to_lowercase().as_str().replace(" ", "-").as_str()
                + ".mtpck",
        )
        .expect("open");
        let mut body = String::new();

        pack_file.read_to_string(&mut body).expect("read_to_string");

        toml::from_str::<Pack>(&body).expect("from_string")
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

        let (mod_version, game_versions) = self.fetch_mod(mod_slug, client);

        self.mods.insert(
            mod_slug.to_string(),
            toml::Value::try_from(&mod_version).expect("try_from"),
        );
        println!(
            "Found mod '{}' and added it to pack",
            mod_version.name.replace("\"", "")
        );
        for dependency in mod_version.dependencies {
            let dep_slug = client
                .get_project(&dependency.project_id)
                .expect("get_project_info")
                .slug;
            if dependency.dependency_type == "required" && !self.mods.contains_key(&dep_slug) {
                println!("Dependency: ");

                self.add_mod(&dep_slug, client);
            }
        }
        game_versions
    }

    /// Downloads all mods from the pack to the download path given in the configuration.
    /// If a download fails once we try and update the mod entry in the pack and redo the download
    /// once.
    pub fn install(&mut self, client: &ApiClient, config: &Configuration) {
        for (key, value) in self.mods.clone() {
            let mod_version: PackMod = value.clone().try_into().expect("try_into");
            let dl_path = config.install_path.clone().unwrap() + &mod_version.file_name;
            println!("Downloading '{key}' to '{dl_path}' ");
            match client.download_file(&dl_path, &mod_version.file_url, &mod_version.sha512) {
                Ok(_) => {}
                Err(_) => {
                    println!(
                        "Downloading '{key}' failed. Update pack entry to resolve possible errors and try again?"
                    );
                    if confirm_input() {
                        let (fetched, _) = self.fetch_mod(&key, client);
                        self.mods.insert(
                            key.to_string(),
                            toml::Value::try_from(&fetched).expect("try_from"),
                        );
                        self.save(config);
                        println!("Retry Downloading '{key}' to '{dl_path}'");
                        client
                            .download_file(&dl_path, &fetched.file_url, &fetched.sha512)
                            .expect("download_file");
                    } else {
                        println!("Could not download '{key}'");
                    }
                }
            };
        }
    }

    /// Get a single Pack mod with its Minecraft Versions
    fn fetch_mod(&self, mod_slug: &String, client: &ApiClient) -> (PackMod, Vec<MCVersion>) {
        let project_version = client
            .get_project_version(mod_slug, &self.version_info)
            .expect("get_project_version");
        (
            PackMod {
                name: project_version.name,
                verstion_type: project_version.version_type,
                version_number: project_version.version_number,
                file_url: project_version.files[0].url.clone(),
                sha512: project_version.files[0].hashes["sha512"]
                    .to_string()
                    .replace("\"", ""),
                file_name: project_version.files[0].filename.clone(),
                dependencies: project_version.dependencies,
            },
            project_version.game_versions,
        )
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
            self.version_info.loader
        )
    }
}

pub fn list_packs(config: Configuration) {
    let dirs = match read_dir(&config.pack_path) {
        Ok(d) => d,
        Err(e) => {
            panic!("{}", e)
        }
    };

    for entry in dirs {
        match entry {
            Ok(entry) => println!(
                "{}",
                Pack::open(
                    &entry
                        .path()
                        .file_stem()
                        .expect("file_stem")
                        .to_owned()
                        .into_string()
                        .expect("into_string"),
                    &config
                )
            ),
            Err(error) => panic!("{error}"),
        }
    }
}
