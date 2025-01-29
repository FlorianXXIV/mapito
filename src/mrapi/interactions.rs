use colored::Colorize;
use reqwest::{blocking::Client, Url};
use serde_json::Value;

use super::{constants::{API_URL, PROJECT, QUERY, SEARCH, VERSION}, defines::{SearchResp, Version, LOADER, VT}};

pub fn search_package(client: &Client, query: String, staging: usize) {
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

pub fn request_api(client: &Client, staging: usize, endpoint: &String) -> Value {
    let query = Url::parse(&(API_URL[staging].to_owned() + endpoint)).unwrap();

    serde_json::from_str(&client.get(query).send().unwrap().text().unwrap()).unwrap()
}

pub fn get_dl_url(
    dl_id: String,
    client: &Client,
    mc_ver: String,
    vt: VT,
    loader: &LOADER,
    staging: usize,
) -> Result<Version, String> {
    let versions: Vec<Version> = serde_json::from_value(request_api(client, staging, &(PROJECT.to_owned() + "/" + &dl_id + VERSION))).expect("from_value");
    let mut dl_version: Option<Version> = None;
    if mc_ver.is_empty() {
        let mut latest_version: Option<Version> = None;
        for version in versions {
            if version.loaders.iter().any(|e| *e == *loader) {
                latest_version = Some(version.clone());
                break;
            }
        };
        if latest_version.is_none() {
            return Err("Loader not available".to_string());
        }
        dl_version = latest_version;
    } else {
            for version in versions {
                if version
                    .game_versions
                    .iter()
                    .any(|e| e.to_string() == mc_ver)
                    && version.version_type == vt
                    && version.loaders.iter().any(|e| *e == *loader)
                {
                    dl_version = Some(version.clone());
                    break;
                }
            }
    }
    if dl_version.is_none() {
        return Err("Did not find Project".to_string());
    }
    Ok(dl_version.expect("Unknown Error"))
}

