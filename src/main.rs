mod client;
mod config;
mod mrapi;
mod pack;

use std::{
    io::{self, stdin},
    str::FromStr,
};

use crate::client::Downloader;

use argparse::{ArgumentParser, Store, StoreConst, StoreOption};
use config::{configure, Configuration};
use mrapi::{
    defines::{Version, LOADER, VT},
    interactions::{get_project_version, print_project_info, search_package},
};
use pack::{create_pack, install_pack};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MVDescriptor {
    mc_ver: String,
    version_types: Vec<VT>,
    loader: LOADER,
}

#[derive(Debug, Clone)]
enum PackAction {
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

fn main() {
    //variables set by arguments
    let mut config = configure().expect("configure");
    let mut search: String = String::new();
    let mut dl_id: String = String::new();
    let mut project_slug: String = String::new();
    let mut pack_action: Option<PackAction> = None;
    //argument parser arg/opt setup
    {
        let mut parser = ArgumentParser::new();

        parser.set_description(
            "A commandline tool to interact with the modrinth \
            database, it allows you to search for projects on modrinth \
            and download them for a minecraft vesion of you choosing into a \
            folder specified by you.",
        );

        parser.refer(&mut config.staging).add_option(
            &["-S", "--staging"],
            StoreConst(0),
            "If set, use the \
                modrinth staging server rather than the normal api server. Used \
                for testing",
        );

        parser.refer(&mut search).add_option(
            &["-s", "--search"],
            Store,
            "Search the Modrinth database for a certain project/mod.",
        );

        parser.refer(&mut config.download_path).add_option(
            &["-p", "--path"],
            Store,
            "The path where you want to download any files to. Default: \
            ~/Downloads",
        );

        parser.refer(&mut dl_id).add_option(
            &["-d", "--download"],
            Store,
            "Download the mod given by it's ID/Slug.",
        );

        parser.refer(&mut config.mc_ver).add_option(
            &["-v", "--mc-ver"],
            Store,
            "Set the Minecraft version that you want to download the Mod for.",
        );

        parser.refer(&mut config.release_type).add_option(
            &["--version-type"],
            Store,
            "Chose verion type, one of: release, beta, alpha",
        );

        parser.refer(&mut config.pack_path).add_option(
            &["--pack-path"],
            Store,
            "Path where pack files are stored",
        );

        parser.refer(&mut config.loader).add_option(
            &["-l", "--loader"],
            Store,
            "The modloader to be used with the mod",
        );

        parser.refer(&mut project_slug).add_option(
            &["-i", "--project-info"],
            Store,
            "Get information about the specified project.",
        );

        parser.refer(&mut pack_action).add_option(
            &["-P", "--pack"],
            StoreOption,
            "Different interactions with Packs, options are\n
            Create, Update, Modify, Install",
        );

        parser.refer(&mut config.install_path).add_option(
            &["--install-path"],
            StoreOption,
            "The path of the modfolder the pack should be installed to."
            );

        parser.parse_args_or_exit();
    }

    let client = Client::new();

    if !search.is_empty() {
        search_package(&client, search, config.staging);
        return;
    }

    if !dl_id.is_empty() {
        let version_desc = MVDescriptor {
            mc_ver: config.mc_ver.clone(),
            version_types: vec![config.release_type.clone()],
            loader: config.loader.clone(),
        };
        let dl_version: Version = get_project_version(&client, config.staging, dl_id, version_desc.clone())
            .expect("get_project_version");
        
        let mut dependencies: Vec<Version> = Vec::new();
        for dependency in dl_version.dependencies {
            let dep_ver = get_project_version(&client, config.staging, dependency.project_id, version_desc.clone()).expect("get_project_version");
            dependencies.push(dep_ver);
        }
        
        let mut dl_size = (dl_version.files[0].size as f64 / 1048576 as f64).to_string();
        dl_size.truncate(6);
        println!(
            "Downloading: {}, {}\ntype: {}, downloads: {}, loader: {:?}\nsize: {} MiB",
            dl_version.name,
            dl_version.version_number,
            dl_version.version_type.to_string(),
            dl_version.downloads,
            dl_version.loaders,
            dl_size
        );

        if confirm_input() {
            println!("Downloading to {}", &config.download_path);
            let filename = dl_version.files[0].filename.as_str();
            let path = &(config.download_path.clone() + "/" + filename);
            let _ = client
                .download_file(
                    path,
                    dl_version.files[0].url.as_str(),
                    dl_version.files[0].hashes["sha512"]
                        .to_string()
                        .replace("\"", "")
                        .as_str(),
                )
                .unwrap();
        } else {
            println!("Aborting");
            return;
        }

        if !dependencies.is_empty() {
            print!("Found the following dependencies:\n {}", dependencies
                .iter()
                .map(|dep| dep.name.clone() + ", " + &(dep.files[0].size as f64 / 1048576 as f64).to_string() + "MB\n")
                .collect::<String>()
                );
            println!("Download these too?");
            if confirm_input() {
                for dep in dependencies {
                    println!("Downloading {}", dep.name);
                    let filename = dep.files[0].filename.as_str();
                    let path = &(config.download_path.clone() + "/" + filename);
                    let _ = client.download_file(path, dep.files[0].url.as_str(), dep.files[0].hashes["sha512"].to_string().replace("\"", "").as_str()).unwrap();
                }
            }
        }
        return;
    }

    if !project_slug.is_empty() {
        print_project_info(&client, config.staging, project_slug);
        return;
    }
    
    match pack_action {
        Some(PackAction::CREATE) => pack_creation_loop(&client, &config),
        Some(PackAction::UPDATE) => todo!(),
        Some(PackAction::MODIFY) => todo!(),
        Some(PackAction::INSTALL) => {
            if config.install_path.is_some() {
                println!("please enter name of pack");
                let i_pack = read_line_to_string();
                install_pack(&client, i_pack, &config);
            } else {
                eprintln!("No install path given")
            }
        },
        None => (),
    }
   
}

fn pack_creation_loop (client: &Client, config: &Configuration) {
    let mut version_desc = MVDescriptor {
        mc_ver: "".to_string(),
        version_types: vec![VT::RELEASE],
        loader: LOADER::FABRIC,
    };
    println!("Please enter the Name of the new Pack:");
    let name = read_line_to_string();
    println!("Please enter the Minecraft version you want the pack to have:");
    version_desc.mc_ver = read_line_to_string();
    println!(
        "Please select what loader you want to use:
        \n[0] - Fabric
        \n[1] - Quilt
        \n[2] - NeoForge
        \n[3] - Forge"
    );
    match read_line_to_string().as_str() {
        "0" | "[0]" => version_desc.loader = LOADER::FABRIC,
        "1" | "[1]" => version_desc.loader = LOADER::QUILT,
        "2" | "[2]" => version_desc.loader = LOADER::NEOFORGE,
        "3" | "[3]" => version_desc.loader = LOADER::FORGE,
        _ => panic!("invalid input"),
    }
    println!(
        "Please type in a list of version types you want to allow:
        \nExample: 'release beta'
        \nAllowed types: 'release' 'beta' 'alpha'"
    );
    version_desc.version_types = read_line_to_string()
        .split_whitespace()
        .map(|vt| VT::from_str(vt).expect("from_str"))
        .collect();
    println!("Please confirm your input:\n Pack Name: {name}\n Minecraft version: {}\n Mod Loader: {}\n version types: {}",
        version_desc.mc_ver,
        version_desc.loader.to_string(),
        version_desc.version_types.iter().map(|vt| vt.to_string() + " ").collect::<String>());
    if !confirm_input() {
        println!("Aborting pack Creation");
        return;
    }
    println!(
        "Now you can search for mods and add them to the pack, you can finish by entering 'q'"
    );
    let mut mods: Vec<String> = Vec::new();
    loop {
        println!("Please enter next query or 'q'.");
        let query = read_line_to_string();
        if query == "q" {
            break;
        } else {
            let slugs = search_package(&client, query, config.staging);
            match slugs {
                Some(sl) => {
                    println!("Select mod from 0 to {} or 'q'", sl.len() - 1);
                    let entry = read_line_to_string();
                    if entry == "q" {
                        continue;
                    } else {
                        let i: usize = entry.parse().expect("parse");
                        mods.push(sl[i].clone());
                    }
                }
                None => {}
            }
        }
    }
    create_pack(
        &client,
        config.staging,
        name,
        version_desc,
        &mut mods,
        &config,
        );
    return;
}

fn confirm_input() -> bool {
    println!("proceed? [Y,n]");
    let stdin = io::stdin();
    let buf = &mut String::new();
    let _ = stdin.read_line(buf);
    let chars: Vec<char> = buf.chars().collect();

    let request: char = match chars.first() {
        Some(c) => *c,
        None => 'y',
    };

    match request {
        'y' | 'Y' | '\n' => true,
        'n' | 'N' => false,
        _ => panic!("invalid option"),
    }
}
/// Reads one line from stdin and returns it as sanitized string
fn read_line_to_string() -> String {
    let buf = &mut String::new();
    stdin().read_line(buf).expect("read_line");
    buf.to_string().replace("\n", "").replace("\"", "")
}
