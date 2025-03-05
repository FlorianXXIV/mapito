mod client;
mod config;
mod mrapi;
mod pack;

use std::{io::{self, stdin}, str::FromStr};

use crate::client::Downloader;

use argparse::{ArgumentParser, Store, StoreConst, StoreTrue};
use colored::Colorize;
use config::configure;
use mrapi::{
    defines::{Version, LOADER, VT},
    interactions::{get_project_version, print_project_info, search_package},
};
use pack::create_pack;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MVDescriptor {
    mc_ver: String,
    version_types: Vec<VT>,
    loader: LOADER,
}

fn main() {
    //variables set by arguments
    let mut config = configure().expect("configure");
    let mut search: String = String::new();
    let mut dl_id: String = String::new();
    let mut project_slug: String = String::new();
    let mut make_pack: bool = false;
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

        parser.refer(&mut make_pack).add_option(
            &["--make-pack"],
            StoreTrue,
            "interactively make a new pack",
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
        let dl_version: Version =
            get_project_version(&client, config.staging, dl_id, version_desc).expect("get_project_version");
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
            let path = &(config.download_path + "/" + filename);
            let _ = client
                .download_file(path, dl_version.files[0].url.as_str())
                .unwrap();
        } else {
            println!("Aborting")
        }
        return;
    }

    if !project_slug.is_empty() {
        print_project_info(&client, config.staging, project_slug);
        return;
    }

    if make_pack {
        let mut version_desc = MVDescriptor {
            mc_ver: "".to_string(),
            version_types: vec![VT::RELEASE],
            loader: LOADER::FABRIC,
        };
        println!("Please enter the Name of the new Pack:");
        let name = read_line_to_string();
        println!("Please enter the Minecraft version you want the pack to have:");
        version_desc.mc_ver = read_line_to_string();
        println!("Please select what loader you want to use:
            \n[0] - Fabric
            \n[1] - Quilt
            \n[2] - NeoForge
            \n[3] - Forge");
        match read_line_to_string().as_str() {
            "0" | "[0]" => version_desc.loader = LOADER::FABRIC,
            "1" | "[1]" => version_desc.loader = LOADER::QUILT,
            "2" | "[2]" => version_desc.loader = LOADER::NEOFORGE,
            "3" | "[3]" => version_desc.loader = LOADER::FORGE,
            _ => panic!("invalid input")
        }
        println!("Please type in a list of version types you want to allow:
            \nExample: 'release beta'
            \nAllowed types: 'release' 'beta' 'alpha'");
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
        println!("Now you can search for mods and add them to the pack, you can finish by entering 'q'");
        let mut mods: Vec<String> = Vec::new();
        loop {
            let query = read_line_to_string();
            if query == "q" {
                break;
            } else {
                let slugs = search_package(&client, query, config.staging);
                match slugs {
                    Some(sl) => {
                        println!("Select mod from 0 to {}", sl.len()-1);
                        let i:usize = read_line_to_string().parse().expect("parse");
                        mods.push(sl[i].clone());
                    },
                    None => {},
                }
            }
        }
        create_pack(&client, config.staging, name, version_desc, &mut mods, &config);
    }
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
