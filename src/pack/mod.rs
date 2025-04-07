use std::fs::{create_dir_all, File};
use std::io::{Read, Write};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use toml::{self, Table};

use crate::client::Downloader;
use crate::{
    config::Configuration,
    mrapi::{
        defines::{Dependency, LOADER, VT},
        interactions::{get_project_info, get_project_version},
    },
    MVDescriptor,
};

#[derive(Deserialize, Serialize, Debug)]
struct Pack {
    name: String,
    version_info: MVDescriptor,
    mods: Table,
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
}

#[derive(Deserialize, Serialize, Debug)]
struct ModVersion {
    name: String,
    verstion_type: VT,
    version_number: String,
    file_url: String,
    file_name: String,
    sha512: String,
}

impl PartialEq for ModVersion {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub fn create_pack(
    client: &Client,
    staging: usize,
    name: String,
    version_desc: MVDescriptor,
    mods: &mut Vec<String>,
    config: &Configuration,
) {
    let mut pack = Pack::new();
    pack.name = name;
    pack.version_info = version_desc.clone();

    let mut dependencies: Vec<Dependency> = Vec::new();

    for mc_mod in mods {
        println!("Looking for {mc_mod}");
        let project_version =
            get_project_version(client, staging, mc_mod.clone(), version_desc.clone())
                .expect("get_project_version");
        let mod_version = ModVersion {
            name: project_version.name,
            verstion_type: project_version.version_type,
            version_number: project_version.version_number,
            file_url: project_version.files[0].url.clone(),
            sha512: project_version.files[0].hashes["sha512"]
                .to_string()
                .replace("\"", ""),
            file_name: project_version.files[0].filename.clone(),
        };
        pack.mods.insert(
            mc_mod.to_string(),
            toml::Value::try_from(&mod_version).expect("try_from"),
        );
        println!(
            "Found mod '{}' and added it to pack",
            mod_version.name.replace("\"", "")
        );
        for dependency in project_version.dependencies {
            if dependency.dependency_type == "required" && !dependencies.contains(&dependency) {
                dependencies.push(dependency);
            }
        }
    }

    for dependency in dependencies {
        let project = get_project_info(client, staging, dependency.project_id.clone())
            .expect("get_project_info");
        let project_version =
            get_project_version(client, staging, dependency.project_id, version_desc.clone())
                .expect("get_project_version");
        let dependency_version = ModVersion {
            name: project_version.name,
            verstion_type: project_version.version_type,
            version_number: project_version.version_number,
            file_url: project_version.files[0].url.clone(),
            sha512: project_version.files[0].hashes["sha512"]
                .to_string()
                .replace("\"", ""),
            file_name: project_version.files[0].filename.clone(),
        };
        if !pack.mods.contains_key(&project.slug) {
            pack.mods.insert(
                project.slug,
                toml::Value::try_from(&dependency_version).expect("try_from"),
            );
            println!(
                "Added dependency '{}' to pack",
                dependency_version.name.replace("\"", "")
            );
        }
    }

    create_dir_all(config.pack_path.clone()).expect("create_dir_all");
    let mut pack_fd = File::create(
        config.pack_path.clone()
            + "/"
            + &pack.name.to_lowercase().as_str().replace(" ", "-")
            + ".mtpck",
    )
    .expect("create");

    write!(
        &mut pack_fd,
        "{}",
        toml::to_string(&pack).expect("to_string")
    )
    .expect("write");
    println!(
        "Created Pack: {}, Minecraft-{}",
        pack.name, pack.version_info.mc_ver
    );
}

pub fn install_pack(client: &Client, name: String, config: &Configuration) {

    let pack = open_pack(&name, config);

    for (key, value) in pack.mods {
        let mod_version: ModVersion = value.try_into().expect("try_into");
        let dl_path = config.install_path.clone().unwrap() + "/" + &mod_version.file_name;
        println!("Downloading '{key}' to '{dl_path}' ");
        let _ = client.download_file(&dl_path, &mod_version.file_url, &mod_version.sha512);
    }
}

pub fn update_pack(client: &Client, name: String, config: &Configuration) {
    let mut pack = open_pack(&name, config);
    println!("Updating mod entries in {name} Modpack.");
    for (key, value) in pack.mods.clone() {
        let mut mod_version: ModVersion = value.try_into().expect("try_into");
        let project_version = get_project_version(client, config.staging, key.clone(), pack.version_info.clone()).expect("get_project_version");
        if mod_version.version_number != project_version.version_number {
            println!("Found new version of {}\nOld: {}\nNew: {}",
                mod_version.name,
                mod_version.version_number,
                project_version.version_number);
            pack.mods.remove::<String>(&key.clone());
            mod_version.name = project_version.name;
            mod_version.verstion_type = project_version.version_type;
            mod_version.version_number = project_version.version_number;
            mod_version.file_url = project_version.files[0].url.clone();
            mod_version.sha512 = project_version.files[0].hashes["sha512"]
                .to_string()
                .replace("\"", "");
            mod_version.file_name = project_version.files[0].filename.clone();
            pack.mods.insert(key, toml::Value::try_from(&mod_version).expect("try_from"));
        } else {
            println!("Mod {} is up to Date.", mod_version.name)
        }
    }
    let pack_name = pack.name.clone();
    save_pack(config, pack);
    println!("To install the Updated mods, use '--pack install' for {pack_name}");
}

/// open the pack file for the given modpack and return Pack object
fn open_pack(name: &String, config: &Configuration) -> Pack {
    let mut pack_file = File::open(config.pack_path.clone() + "/"
        + name.clone().to_lowercase().as_str().replace(" ", "-").as_str()
        + ".mtpck")
        .expect("open");
    let mut body = String::new();

    pack_file.read_to_string(&mut body).expect("read_to_string");

    let pack = toml::from_str::<Pack>(&body).expect("from_string");

    pack
}

fn save_pack(config: &Configuration, pack: Pack) {
    println!("Saving Changes for {}", pack.name);
    create_dir_all(config.pack_path.clone()).expect("create_dir_all");
    let mut pack_fd = File::create(
        config.pack_path.clone()
            + "/"
            + &pack.name.to_lowercase().as_str().replace(" ", "-")
            + ".mtpck",
    )
    .expect("create");

    write!(
        &mut pack_fd,
        "{}",
        toml::to_string(&pack).expect("to_string")
    )
    .expect("write");
}
