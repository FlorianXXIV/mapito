use colored::Colorize;
use reqwest::{blocking::Client, Url};
use serde_json::Value;

use crate::MVDescriptor;

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

pub fn get_project_version(
    client: &Client,
    staging: usize,
    project_slug: String,
    version_desc: MVDescriptor
    ) -> Result<Version, String>{
    let mut project_version: Option<Version> = None;
    let versions: Vec<Version> = serde_json::from_value(
            request_api(
                client,
                staging,
                &(PROJECT.to_owned() + "/" + &project_slug + VERSION)
                ).expect("request_api")
        ).expect("from_value");
    for version in versions {
        if (
            version.game_versions.contains(&version_desc.mc_ver) 
            || version_desc.mc_ver == "latest"
            )
            && version_desc.version_types.contains(&version.version_type)
            && version.loaders.contains(&version_desc.loader){
            project_version = Some(version.clone());
            break;
        }
    }

    if project_version.is_none() {
        return Err("Specified Mod Version not found".to_string());
    }

    Ok(project_version.expect("Unknown Error"))
}

