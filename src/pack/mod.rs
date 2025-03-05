use std::fs::{create_dir_all, File};
use std::io::Write;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use toml::{self, Table};

use crate::{config::{self, Configuration}, mrapi::{defines::{Dependency, LOADER, VT}, interactions::{get_project_info, get_project_version}}, MVDescriptor};

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
                loader: LOADER::FABRIC 
            },
            mods: Table::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct ModVersion {
    name: String,
    verstion_type: VT,
    file_url: String,
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
        let project_version = get_project_version(client, staging, mc_mod.clone(), version_desc.clone()).expect("get_project_version");
        let mod_version = ModVersion {
            name: project_version.name,
            verstion_type: project_version.version_type,
            file_url: project_version.files[0].url.clone(),
            sha512: project_version.files[0].hashes["sha512"].to_string().replace("\"", "")
        };
        pack.mods.insert(mc_mod.to_string(), toml::Value::try_from(&mod_version).expect("try_from"));
        println!("Found mod '{}' and added it to pack", mod_version.name.replace("\"", ""));
        for dependency in project_version.dependencies {
            if dependency.dependency_type == "required" 
            && !dependencies.contains(&dependency){
                dependencies.push(dependency);
            }
        }
    }

    for dependency in dependencies {
        let project = get_project_info(client, staging, dependency.project_id.clone()).expect("get_project_info");
        let project_version = get_project_version(
            client,
            staging,
            dependency.project_id,
            version_desc.clone()).expect("get_project_version");
        let dependency_version = ModVersion {
            name: project_version.name,
            verstion_type: project_version.version_type,
            file_url: project_version.files[0].url.clone(),
            sha512: project_version.files[0].hashes["sha512"].to_string().replace("\"", "")
        };
        if !pack.mods.contains_key(&project.slug) {
            pack.mods.insert(project.slug, toml::Value::try_from(&dependency_version).expect("try_from"));
            println!("Added dependency '{}' to pack", dependency_version.name.replace("\"", ""));
        }
    }

    create_dir_all(config.pack_path.clone()).expect("create_dir_all");
    let mut pack_fd = File::create(config.pack_path.clone() + "/" + &pack.name.to_lowercase().as_str().replace(" ", "_") + ".mtpck").expect("create");

    write!(&mut pack_fd, "{}", toml::to_string(&pack).expect("to_string")).expect("write");
    println!("Created Pack: {}, Minecraft-{}", pack.name, pack.version_info.mc_ver);
}


