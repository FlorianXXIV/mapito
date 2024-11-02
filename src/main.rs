use std::env;

use argparse::{parser, ArgumentParser, Store, StoreConst, StoreTrue};
use reqwest::{blocking::Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const API_URL: [&str; 2] = [
    "https://staging-api.modrinth.com/",
    "https://api.modrinth.com/",
];

#[derive(Debug, Serialize, Deserialize)]
struct SearchResp {
    hits: Vec<Value>,
    offset: i32,
    limit: i32,
    total_hits: i32,
}

fn main() {
    //variables set by arguments
    let mut staging = 1;
    let mut search: String = String::new();
    let mut dl_path: String = env::var("HOME").unwrap() + "/Downloads";
    //argument parser arg/opt setup
    {
        let mut parser = ArgumentParser::new();

        parser.set_description("A commandline tool to interact with the modrinth database, it allows you to search for mods and download them in a folder specified by you.");

        parser.refer(&mut staging)
            .add_option(&["-S", "--staging"], StoreConst(0), "If set, use the modrinth staging server rather than the normal api server. Used for testing");
        parser.refer(&mut search).add_option(
            &["-s", "--search"],
            Store,
            "Search the Modrinth database for a certain project/mod.",
        );
        parser.refer(&mut dl_path)
            .add_option(&["-p","--path"], Store, "The path where you want to download any files to. Default: ~/Downloads");

        parser.parse_args_or_exit();
    }

    let client = Client::new();

    if !search.is_empty() {
        let query = Url::parse_with_params(
            (API_URL[staging].to_owned() + &"v2/search").as_str(),
            &[("query", search)],
        )
        .unwrap();
        let query_response = client
            .get(query)
            .send()
            .unwrap()
            .json::<SearchResp>()
            .unwrap();

        for hit in query_response.hits {
            println!("----\nName: {}, Latest Version: {}\n\n{}\n\nAuthor: {}\nId: {}\nDownloads: {}\n\n",hit["title"],hit["versions"][0],hit["description"], hit["author"], hit["project_id"], hit["downloads"]);
        }
    }
}
