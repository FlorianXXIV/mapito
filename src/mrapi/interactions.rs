use colored::Colorize;
use reqwest::{blocking::Client, Url};
use serde_json::Value;

use crate::config::Configuration;

use super::{
    constants::{API_URL, MEMBERS, PROJECT, QUERY, SEARCH, VERSION},
    defines::{Member, Project, SearchResp, Version},
};

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

fn request_api(client: &Client, staging: usize, endpoint: &String) -> Result<Value, serde_json::Error> {
    let query = Url::parse(&(API_URL[staging].to_owned() + endpoint)).unwrap();

    Ok(serde_json::from_str(
        &client
            .get(query)
            .send()
            .expect("send")
            .text()
            .expect("text")
        ))?
}

pub fn get_dl_url(
    dl_id: String,
    client: &Client,
    config: &Configuration,
) -> Result<Version, String> {
    let versions: Vec<Version> = serde_json::from_value(request_api(
        client,
        config.staging,
        &(PROJECT.to_owned() + "/" + &dl_id + VERSION),
    ).expect("request_api")
    ).expect("from_value");
    let mut dl_version: Option<Version> = None;
    if config.mc_ver == "latest" {
        let mut latest_version: Option<Version> = None;
        for version in versions {
            if version.loaders.iter().any(|e| *e == config.loader) {
                latest_version = Some(version.clone());
                break;
            }
        }
        if latest_version.is_none() {
            return Err("Loader not available".to_string());
        }
        dl_version = latest_version;
    } else {
        for version in versions {
            if version
                .game_versions
                .iter()
                .any(|e| e.to_string() == config.mc_ver)
                && version.version_type == config.release_type
                && version.loaders.iter().any(|e| *e == config.loader)
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

pub fn print_project_info(client: &Client, staging: usize, project_slug: String) {
    let project: Project = serde_json::from_value(request_api(
        client,
        staging,
        &(PROJECT.to_string() + "/" + &project_slug),
    ).expect("request_api"))
    .expect("from_value");
    let members: Vec<Member> = serde_json::from_value(request_api(
        client,
        staging,
        &(PROJECT.to_string() + "/" + &project_slug + MEMBERS),
    ).expect("request_api"))
    .expect("from_value");
    println!(
        "Project: {}, latest-{}, {}\n {}\n\n Released: {}\n Last Updated: {} \n \
        loaders: {}\n supported versions: \n{} license: {}\n source: {}\n members:\n{}",
        project.title,
        project.game_versions.last().expect("last"),
        project.project_type.green(),
        project.description,
        project.published.yellow(),
        project.updated.yellow(),
        project.loaders.iter().map(|e| e.to_string() + ",").collect::<String>(),
        project.game_versions
            .iter().rev().take(10)
            .map(|e| "  ".to_string()  + &e.to_string() + "\n")
            .collect::<String>(),
        project.license.name,
        match project.source_url {
            Some(v) => {v.bright_blue()},
            None => {"none".to_string().red()},
        },
        members
            .iter()
            .map(|mem| "  ".to_string() + &mem.user.username.clone() + ", " + &mem.role + "\n")
            .collect::<String>(),
    );
}

