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
    //argument parser arg/opt setup
    {
        let mut parser = ArgumentParser::new();

        parser.refer(&mut staging)
            .add_option(&["-S", "--staging"], StoreConst(0), "If set, use the modrinth staging server rather than the normal api server. Used for testing");
        parser.refer(&mut search).add_option(
            &["-s", "--search"],
            Store,
            "Search the Modrinth database for a certain project/mod.",
        );

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

        println!("Search Result: \n {query_response:?}");
    }
}
