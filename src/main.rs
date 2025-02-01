mod client;
mod config;
mod mrapi;

use std::io;

use crate::client::Downloader;

use argparse::{ArgumentParser, Store, StoreConst};
use config::configure;
use mrapi::{
    defines::Version,
    interactions::{get_dl_url, print_project_info, search_package},
};
use reqwest::blocking::Client;

fn main() {
    //variables set by arguments
    let mut config = configure().expect("configure");
    let mut search: String = String::new();
    let mut dl_id: String = String::new();
    let mut project_slug: String = String::new();
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

        parser.parse_args_or_exit();
    }

    let client = Client::new();

    if !search.is_empty() {
        search_package(&client, search, config.staging);
    }

    if !dl_id.is_empty() {
        let dl_version: Version =
            get_dl_url(dl_id, &client, &config).expect("get_dl_url");
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
    }

    if !project_slug.is_empty() {
        print_project_info(&client, config.staging, project_slug);
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
