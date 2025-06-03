use std::fs::{create_dir_all, File};
use std::io::{Write};

use pack::Pack;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use toml::{self};

use crate::mc_info::VT;
use crate::util::error::ApiError;
use crate::{
    config::Configuration,
    mrapi::
        interactions::get_project_version
    ,
    MVDescriptor,
};

pub mod pack;

#[derive(Deserialize, Serialize, Debug)]
struct PackMod {
    name: String,
    verstion_type: VT,
    version_number: String,
    file_url: String,
    file_name: String,
    sha512: String,
}

impl PartialEq for PackMod {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub fn create_pack(
    client: &Client,
    staging: usize,
    name: String,
    version_desc: MVDescriptor,
    mods: &Vec<String>,
    config: &Configuration,
) {
    let mut pack = Pack::new();
    pack.name = name;
    pack.version_info = version_desc.clone();

    for mc_mod in mods {
        pack.add_mod(mc_mod, client, staging);
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

pub fn update_pack(client: &Client, name: String, config: &Configuration) -> Result<(), ApiError> {
    let mut pack = Pack::open(&name, config);
    println!("Updating mod entries in {name} Modpack.");
    for (key, value) in pack.mods.clone() {
        let mut mod_version: PackMod = value.try_into().expect("try_into");
        let project_version = get_project_version(
            client,
            config.staging,
            key.clone(),
            pack.version_info.clone(),
        )?;
        if mod_version.version_number != project_version.version_number {
            println!(
                "Found new version of {}\nOld: {}\nNew: {}",
                mod_version.name, mod_version.version_number, project_version.version_number
            );
            pack.mods.remove::<String>(&key.clone());
            mod_version.name = project_version.name;
            mod_version.verstion_type = project_version.version_type;
            mod_version.version_number = project_version.version_number;
            mod_version.file_url = project_version.files[0].url.clone();
            mod_version.sha512 = project_version.files[0].hashes["sha512"]
                .to_string()
                .replace("\"", "");
            mod_version.file_name = project_version.files[0].filename.clone();
            pack.mods
                .insert(key, toml::Value::try_from(&mod_version).expect("try_from"));
        } else {
            println!("Mod {} is up to Date.", mod_version.name)
        }
    }
    let pack_name = pack.name.clone();
    pack.save(config);
    println!("To install the Updated mods, use '--pack install' for {pack_name}");
    Ok(())
}

