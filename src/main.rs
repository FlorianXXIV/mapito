mod argparse;
mod cli;
mod client;
mod config;
mod mc_info;
mod mrapi;
mod pack;
mod util;

use std::{env::var, os::unix::process::CommandExt, process::Command};

use crate::{
    cli::interactions::{list_multi_select, list_select},
    client::Downloader,
    config::config_path,
    mc_info::LOADERS,
    pack::pack::list_packs,
    util::byte_to_readable,
};

use argparse::Commands;
use clap::Parser;
use cli::{
    input::{confirm_input, query_pack, read_line_to_string},
    interactions::{prompt_for, search_mods},
};
use config::{configure, Configuration};
use mc_info::{Loader, MCVersion, MVDescriptor, VT};
use mrapi::{client::ApiClient, defines::Version};
use pack::{
    create_pack,
    pack::{Pack, PackAction},
    update_pack,
};

fn main() {
    //variables set by arguments
    let config = configure().expect("configure");
    let project_slug: String = String::new();
    let parser = argparse::Arguments::parse();
    let api_client = ApiClient::new(parser.staging);

    if let Some(search) = parser.search {
        api_client
            .search(
                &search,
                None,
                None,
                &Some(vec![
                    vec![("versions".to_string(), config.mc_ver.to_string())],
                    vec![("categories".to_string(), config.loader.to_string())],
                ]),
            )
            .expect("search");
        return;
    }

    if let Some(dl_id) = parser.download {
        let version_desc = MVDescriptor {
            mc_ver: config.mc_ver,
            version_types: vec![config.release_type],
            loader: config.loader,
        };
        let dl_version: Version = match api_client.get_project_version(&dl_id, &version_desc) {
            Ok(v) => v,
            Err(e) => {
                println!("get_project_version: {}", e);
                return;
            }
        };

        let mut dependencies: Vec<Version> = Vec::new();
        for dependency in dl_version.dependencies {
            let dep_ver =
                match api_client.get_project_version(&dependency.project_id, &version_desc) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("get_project_version: {}", e);
                        continue;
                    }
                };
            dependencies.push(dep_ver);
        }

        let dl_size = byte_to_readable(dl_version.files[0].size);
        println!(
            "Downloading: {}, {}\ntype: {}, downloads: {}, loader: {:?}\nsize: {}",
            dl_version.name,
            dl_version.version_number,
            dl_version.version_type,
            dl_version.downloads,
            dl_version.loaders,
            dl_size
        );

        if confirm_input() {
            println!("Downloading to {}", &config.download_path);
            let filename = dl_version.files[0].filename.as_str();
            let path = &(config.download_path.clone() + "/" + filename);
            api_client
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
            print!(
                "Found the following dependencies:\n {}",
                dependencies
                    .iter()
                    .map(|dep| dep.name.clone()
                        + ", "
                        + &byte_to_readable(dep.files[0].size)
                        + "\n")
                    .collect::<String>()
            );
            println!("Download these too?");
            if confirm_input() {
                for dep in dependencies {
                    println!("Downloading {}", dep.name);
                    let filename = dep.files[0].filename.as_str();
                    let path = &(config.download_path.clone() + "/" + filename);
                    api_client
                        .download_file(
                            path,
                            dep.files[0].url.as_str(),
                            dep.files[0].hashes["sha512"]
                                .to_string()
                                .replace("\"", "")
                                .as_str(),
                        )
                        .unwrap();
                }
            }
        }
        return;
    }

    if !project_slug.is_empty() {
        api_client.print_project_info(&project_slug);
        return;
    }

    match &parser.command {
        Some(Commands::Pack(action)) => match action.pack_action {
            PackAction::Create => pack_creation_loop(&api_client, &config),
            PackAction::Update => {
                println!("Please enter the name of the Pack you want to Update");
                let name = read_line_to_string();
                update_pack(&api_client, name, &config).expect("update_pack");
            }
            PackAction::Modify => pack_modification_loop(&api_client, &config),
            PackAction::Install => {
                if config.install_path.is_some() {
                    let mut pack = query_pack(PackAction::Install, &config);
                    pack.install(&api_client, &config);
                } else {
                    eprintln!("No install path given")
                }
            }
            PackAction::Remove => {
                let pack = query_pack(PackAction::Remove, &config);
                pack.remove(&config);
            }
            PackAction::List => {
                list_packs(config);
            }
        },
        Some(Commands::Config { info }) => {
            if *info {
                println!("{}", config);
            } else {
                eprintln!(
                    "Could not Start Editor: {}",
                    Command::new(var("EDITOR").unwrap_or("nano".to_string()))
                        .args(config_path())
                        .exec()
                );
            }
        }
        None => (),
    }
}

fn pack_creation_loop(client: &ApiClient, config: &Configuration) {
    let mut version_desc = MVDescriptor {
        mc_ver: MCVersion::new(),
        version_types: vec![VT::Release],
        loader: Loader::Fabric,
    };

    let abort_msg = "Aborting pack creation.";

    println!("Please enter the Name of the new Pack:");
    let name = read_line_to_string();
    version_desc.mc_ver = match prompt_for("Please enter the Minecraft version of this pack") {
        Some(ver) => ver,
        None => {
            println!("{abort_msg}");
            return;
        }
    };
    version_desc.loader = match list_select("Select a Modloader", LOADERS) {
        Some(loader) => loader,
        None => {
            println!("{abort_msg}");
            return;
        }
    };
    version_desc.version_types =
        match list_multi_select("Choose Version Types", &[VT::Release, VT::Beta, VT::Alpha]) {
            Some(vt) => vt,
            None => {
                println!("{abort_msg}");
                return;
            }
        };
    println!("Please confirm your input:\n Pack Name: {name}\n Minecraft version: {}\n Mod Loader: {}\n version types: {}",
        version_desc.mc_ver,
        version_desc.loader,
        version_desc.version_types.iter().map(|vt| vt.to_string() + " ").collect::<String>());
    if !confirm_input() {
        println!("Aborting pack Creation");
        return;
    }
    println!(
        "Now you can search for mods and add them to the pack, you can finish by entering 'q'"
    );
    let mods: Vec<String> = search_mods(client, Some(&version_desc));

    create_pack(client, name, version_desc, &mods, config);
}

fn pack_modification_loop(client: &ApiClient, config: &Configuration) {
    let mut pack = query_pack(PackAction::Modify, config);
    loop {
        println!("{}", pack,);
        match prompt_for::<char>("choose a category to modify:\n0 - Name\n1 - Version Info\n\tMinecraft Version\n\tVersion Types\n\tLoader\n2 - Mods\n") {
            Some('0') => {
                pack.remove(config);
                match prompt_for::<String>("Enter a new name for the Pack.") {
                    Some(name) => pack.name = name,
                    None => println!("Name not changed."),
                };
                pack.save(config);
                return;
            }
            Some('1') => {
                let true_name = pack.name.clone();
                pack.name += "_tmp";
                loop {
                    println!("What do you want to change?");
                    println!("  0 - Minecraft Version: {}", pack.version_info.mc_ver);
                    println!(
                        "  1 - Version Types: {}",
                        pack.version_info
                            .version_types
                            .iter()
                            .map(|vt| vt.to_string() + " ")
                            .collect::<String>()
                    );
                    println!("  2 - Loader: {}", pack.version_info.loader);
                    match prompt_for::<char>("") {
                        Some('0') => {
                            match prompt_for::<MCVersion>("enter a new Minecraft version for the Pack.") {
                                Some(ver) => {pack.version_info.mc_ver = ver},
                                None => {
                                    println!("Version not changed.");
                                },
                            };
                        }
                        Some('1') => {
                            println!("enter new version types for the Pack.");
                            match list_multi_select("Enter new version types for the Pack.", &[VT::Release, VT::Beta, VT::Alpha]) {
                                Some(vt) => {pack.version_info.version_types = vt},
                                None => println!("Version Types not changed"),
                            };
                        }
                        Some('2') => {
                            match list_select(
                                "Please enter the loader you want to change to",
                                &[
                                    Loader::Fabric,
                                    Loader::Quilt,
                                    Loader::Neoforge,
                                    Loader::Forge
                                ]
                            ) {
                                Some(loader) => pack.version_info.loader = loader,
                                None => println!("Loader not changed."),
                            };
                        }
                        None => break,
                        _ => println!("unexpected input"),
                    }
                }
                pack.save(config);
                println!("updating mods.");
                match update_pack(client, pack.name.clone(), config) {
                    Ok(_) => {
                        pack = Pack::open(&pack.name, config);
                        pack.remove(config);
                        pack.name = true_name;
                        pack.save(config);
                    }
                    Err(_) => {
                        pack.remove(config);
                        pack.name = true_name;
                    }
                };
                pack = Pack::open(&pack.name, config);
            }
            Some('2') => loop {
                pack.list_mods();
                println!("Choose an Action:");
                println!("  0 - add mods");
                println!("  1 - remove a mod");
                match prompt_for::<char>("") {
                    Some('0') => {
                        let mods = search_mods(client,  Some(&pack.version_info));
                        for item in mods {
                            pack.add_mod(&item, client);
                        }
                        pack.save(config);
                    }
                    Some('1') => {
                        println!("Enter which mod to remove:");
                        pack.mods.remove(&read_line_to_string());
                        pack.save(config);
                        pack = Pack::open(&pack.name, config);
                    }
                    None => break,
                    _ => println!("unexpected input"),
                }
            },
            None => return,
            _ => println!("unexpected input"),
        }
    }
}
