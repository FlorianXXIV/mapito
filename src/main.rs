mod cli;
mod client;
mod config;
mod mc_info;
mod mrapi;
mod pack;
mod util;

use crate::client::Downloader;

use argparse::{ArgumentParser, Store, StoreConst, StoreOption};
use cli::{
    input::{confirm_input, query_pack, read_line_to_string},
    interactions::{prompt_for, prompt_multiple, search_mods},
};
use config::{configure, Configuration};
use mc_info::{MCVersion, MVDescriptor, LOADER, VT};
use mrapi::{
    defines::Version,
    interactions::{get_project_version, print_project_info, search_package},
};
use pack::{
    create_pack,
    pack::{Pack, PackAction},
    update_pack,
};
use reqwest::blocking::Client;

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
            "The path of the modfolder the pack should be installed to.",
        );

        parser.parse_args_or_exit();
    }

    let client = Client::new();

    if !search.is_empty() {
        search_package(&client, &search, config.staging, None, None);
        return;
    }

    if !dl_id.is_empty() {
        let version_desc = MVDescriptor {
            mc_ver: config.mc_ver,
            version_types: vec![config.release_type.clone()],
            loader: config.loader.clone(),
        };
        let dl_version: Version =
            match get_project_version(&client, config.staging, dl_id, version_desc.clone()) {
                Ok(v) => v,
                Err(e) => {
                    println!("{}", e.to_string());
                    return;
                }
            };

        let mut dependencies: Vec<Version> = Vec::new();
        for dependency in dl_version.dependencies {
            let dep_ver = match get_project_version(
                &client,
                config.staging,
                dependency.project_id,
                version_desc.clone(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    println!("get_project_version: {}", e.to_string());
                    continue;
                }
            };
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
            print!(
                "Found the following dependencies:\n {}",
                dependencies
                    .iter()
                    .map(|dep| dep.name.clone()
                        + ", "
                        + &(dep.files[0].size as f64 / 1048576 as f64).to_string()
                        + "MB\n")
                    .collect::<String>()
            );
            println!("Download these too?");
            if confirm_input() {
                for dep in dependencies {
                    println!("Downloading {}", dep.name);
                    let filename = dep.files[0].filename.as_str();
                    let path = &(config.download_path.clone() + "/" + filename);
                    let _ = client
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
        print_project_info(&client, config.staging, project_slug);
        return;
    }

    match pack_action {
        Some(PackAction::CREATE) => pack_creation_loop(&client, &config),
        Some(PackAction::UPDATE) => {
            println!("Please enter the name of the Pack you want to Update");
            let name = read_line_to_string();
            update_pack(&client, name, &config).expect("update_pack");
        }
        Some(PackAction::MODIFY) => pack_modification_loop(&client, &config),
        Some(PackAction::INSTALL) => {
            if config.install_path.is_some() {
                let pack = query_pack(PackAction::INSTALL, &config);
                pack.install(&client, &config);
            } else {
                eprintln!("No install path given")
            }
        }
        Some(PackAction::REMOVE) => {
            let pack = query_pack(PackAction::REMOVE, &config);
            pack.remove(&config);
        }
        None => (),
    }
}

fn pack_creation_loop(client: &Client, config: &Configuration) {
    let mut version_desc = MVDescriptor {
        mc_ver: MCVersion::new(),
        version_types: vec![VT::RELEASE],
        loader: LOADER::FABRIC,
    };

    let abort_msg = "Aborting pack creation.";

    println!("Please enter the Name of the new Pack:");
    let name = read_line_to_string();
    version_desc.mc_ver = match prompt_for("Please enter the Minecraft version of this pack") {
        Some(ver) => ver,
        None => {
            println!("{}", abort_msg);
            return;
        }
    };
    version_desc.loader = match prompt_for("Please enter what loader you want to use") {
        Some(loader) => loader,
        None => {
            println!("{}", abort_msg);
            return;
        }
    };
    version_desc.version_types = prompt_multiple("Please enter one of 'release' 'beta' 'alpha'");
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
    let mut mods: Vec<String> = search_mods(client, config);

    create_pack(&client, config.staging, name, version_desc, &mut mods, &config);
    return;
}

fn pack_modification_loop(client: &Client, config: &Configuration) {
    let mut pack = query_pack(PackAction::MODIFY, config);
    loop {
        println!("{}", pack.to_string(),);
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
                pack.name = pack.name + "_tmp";
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
                    println!("  2 - Loader: {}", pack.version_info.loader.to_string());
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
                            pack.version_info.version_types = prompt_multiple("Enter new version types for the Pack.");
                        }
                        Some('2') => {
                            match prompt_for::<LOADER>("Please enter the loader you want to change to") {
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
                        let mods = search_mods(client, config);
                        for item in mods {
                            pack.add_mod(&item, client, config.staging);
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
