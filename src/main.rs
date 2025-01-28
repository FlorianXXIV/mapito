mod client;
mod config;

use core::panic;
use std::{char, io, u64};

use crate::client::Downloader;
use colored::Colorize;

use argparse::{ArgumentParser, Store, StoreConst};
use config::{configure, VT};
use reqwest::{blocking::Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const API_URL: [&str; 2] = [
    "https://staging-api.modrinth.com/v2",
    "https://api.modrinth.com/v2",
];
//API ENDPOINTS
const SEARCH: &str = "/search";
const PROJECT: &str = "/project";
const VERSION: &str = "/version";
//API PARAMS
const QUERY: &str = "query";

#[derive(Debug, Serialize, Deserialize)]
struct SearchResp {
    hits: Vec<Value>,
    offset: i32,
    limit: i32,
    total_hits: i32,
}

fn main() {
    //variables set by arguments
    let (mut vt, mut dl_path, mut pack_path) = configure().expect("configure");
    let mut staging = 1;
    let mut search: String = String::new();
    let mut dl_id: String = String::new();
    let mut mc_ver: String = String::new();
    //argument parser arg/opt setup
    {
        let mut parser = ArgumentParser::new();

        parser.set_description(
            "A commandline tool to interact with the modrinth \
            database, it allows you to search for mods and download them in a \
            folder specified by you.",
        );

        parser.refer(&mut staging).add_option(
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

        parser.refer(&mut dl_path).add_option(
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

        parser.refer(&mut mc_ver).add_option(
            &["-v", "--mc-ver"],
            Store,
            "Set the Minecraft version that you want to download the Mod for.",
        );

        parser.refer(&mut vt).add_option(
            &["--version-type"],
            Store,
            "Chose verion type, one of: release, beta, alpha",
        );

        parser.refer(&mut pack_path).add_option(
            &["--pack-path"],
            Store,
            "Path where pack files are stored",
        );

        parser.parse_args_or_exit();
    }

    let client = Client::new();

    if !search.is_empty() {
        search_package(&client, search, staging);
    }

    if !dl_id.is_empty() {
        let dl_version: Value =
            get_dl_url(dl_id, &client, mc_ver, vt, staging).expect("get_dl_url");
        let mut dl_size = (dl_version["files"][0]["size"].as_f64().unwrap() / 1048576 as f64).to_string();
        dl_size.truncate(6);
        println!(
            "Downloading: {}, {}\ntype: {},downloads: {}\nsize: {} MiB",
            dl_version["name"].to_string(),
            dl_version["version_number"].to_string(),
            dl_version["version_type"].to_string(),
            dl_version["downloads"].to_string(),
            dl_size
        );

        if confirm_input() {
            println!("Downloading to {}", &dl_path);
                let filename = dl_version["files"][0]["filename"].as_str().unwrap();
                let path = &(dl_path + "/" + filename);
                let _ = client
                    .download_file(path, dl_version["files"][0]["url"].as_str().unwrap())
                    .unwrap();
        } else {
            println!("Aborting")
        }
    }
}

fn search_package(client: &Client, query: String, staging: usize) {
    let query = Url::parse_with_params(
        (API_URL[staging].to_owned() + SEARCH).as_str(),
        &[(QUERY, query)],
    )
    .unwrap();
    let query_response = client
        .get(query)
        .send()
        .unwrap()
        .json::<SearchResp>()
        .unwrap();

    for hit in query_response.hits {
        let versions = hit["versions"].as_array().unwrap();
        let latest = versions[versions.len() - 1].clone();
        println!(
            "{}|{},{}, MC-{}, by: {}, downloads: {}\n{}\n",
            hit["slug"].to_string().replace("\"", "").green(),
            hit["title"].to_string().replace("\"", ""),
            hit["project_type"].to_string().replace("\"", ""),
            latest.to_string().replace("\"", ""),
            hit["author"].to_string().replace("\"", ""),
            hit["downloads"].to_string().replace("\"", ""),
            hit["description"]
                .to_string()
                .replace("\"", "")
                .bright_black(),
        );
    }
}

fn get_dl_url(
    dl_id: String,
    client: &Client,
    mc_ver: String,
    vt: VT,
    staging: usize,
) -> Result<Value, &str> {
    let project: Value = request_api(client, staging, &(PROJECT.to_owned() + "/" + &dl_id));
    let versions = project["versions"].as_array().unwrap();
    let mut dl_version: Value = Value::Null;
    if mc_ver.is_empty() {
        let latest_version = versions.last().unwrap().clone();

        dl_version = request_api(
            client,
            staging,
            &(VERSION.to_owned() + "/" + latest_version.as_str().unwrap()),
        );
    } else {
        if project["game_versions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| *e == *mc_ver)
        {
            let versions = request_api(
                client,
                staging,
                &(PROJECT.to_owned() + "/" + &dl_id + VERSION),
            );
            for version in versions.as_array().unwrap().into_iter() {
                if version["game_versions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|e| *e == *mc_ver)
                    && version["version_type"] == vt.to_string()
                {
                    dl_version = version.clone();
                    break;
                }
            }
        } else {
            return Err("Minecraft version not Available!");
        }
    }
    if dl_version.is_null() {
        return Err("Did not find Project");
    }
    Ok(dl_version)
}

fn request_api(client: &Client, staging: usize, endpoint: &String) -> Value {
    let query = Url::parse(&(API_URL[staging].to_owned() + endpoint)).unwrap();

    serde_json::from_str(&client.get(query).send().unwrap().text().unwrap()).unwrap()
}

fn confirm_input() -> bool {
    println!("proceed? [Y,n]");
    let stdin = io::stdin();
    let buf = &mut String::new();
    let _ = stdin.read_line(buf);
    let chars:Vec<char> = buf.chars().collect();

    let request:char = match chars.first() {
        Some(c) => {*c},
        None => {'y'},
    };
        
    match request {
        'y' | 'Y' | '\n' => true,
        'n' | 'N' => false,
        _ => panic!("invalid option"),
    }
}
