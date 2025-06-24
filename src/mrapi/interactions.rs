use colored::Colorize;
use reqwest::{blocking::Client, Url};
use serde_json::Value;

use crate::{util::error::ApiError, MVDescriptor};

use super::{
    constants::{API_URL, FACETS, LIMIT, MEMBERS, OFFSET, PROJECT, QUERY, SEARCH, VERSION},
    defines::{Member, Project, SearchResp, Version},
};

#[deprecated]
pub fn search_package(
    client: &Client,
    query: &String,
    staging: usize,
    limit: Option<usize>,
    offset: Option<usize>,
    facets: &Option<Vec<Vec<(String, String)>>>,
) -> Option<Vec<String>> {
    let par_limit = match limit {
        Some(num) => num.to_string(),
        None => "10".to_owned(),
    };

    let par_offset = match offset {
        Some(num) => num.to_string(),
        None => "0".to_owned(),
    };

    let query = match facets {
        Some(facets) => {
            let mut str_facet:String = "[".to_string();
            for and in facets {
                str_facet += "[";
                for or in and {
                    str_facet += "\"";
                    str_facet += or.0.as_str();
                    str_facet += ":";
                    str_facet += or.1.as_str();
                    str_facet += "\"";
                    if !(or == and.last().unwrap()) {
                        str_facet += ",";
                    }
                }
                str_facet += "]";
                if !(and == facets.last().unwrap()) {
                    str_facet += ",";
                }
            }
            str_facet += "]";
            Url::parse_with_params(
                (API_URL[staging].to_owned() + SEARCH).as_str(),
                &[(QUERY, query), (LIMIT, &par_limit), (OFFSET, &par_offset), (FACETS, &str_facet)],
            )
            .unwrap()
        }
        None => {
            Url::parse_with_params(
                (API_URL[staging].to_owned() + SEARCH).as_str(),
                &[(QUERY, query), (LIMIT, &par_limit), (OFFSET, &par_offset)],
            )
            .unwrap()
        }
    };
    let query_response = match client.get(query).send().unwrap().json::<SearchResp>() {
        Ok(v) => v,
        Err(_) => {
            println!("Query failed.");
            return None;
        }
    };

    let mut slugs: Vec<String> = Vec::new();
    let mut counter = 0;
    for hit in query_response.hits {
        let versions = hit["versions"].as_array().unwrap();
        let latest = versions[versions.len() - 1].clone();
        println!(
            "{counter} {}|{},{}, MC-{}, by: {}, downloads: {}\n{}\n",
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
        counter += 1;
        slugs.push(hit["slug"].to_string().replace("\"", ""));
    }

    Some(slugs)
}

fn request_api(
    client: &Client,
    staging: usize,
    endpoint: &String,
) -> Result<Value, serde_json::Error> {
    let query = Url::parse(&(API_URL[staging].to_owned() + endpoint)).unwrap();

    Ok(serde_json::from_str(
        &client
            .get(query)
            .send()
            .expect("send")
            .text()
            .expect("text"),
    ))?
}

#[deprecated]
pub fn print_project_info(client: &Client, staging: usize, project_slug: String) {
    let project: Project =
        get_project(client, staging, project_slug.clone()).expect("get_project_info");
    let members: Vec<Member> = serde_json::from_value(
        request_api(
            client,
            staging,
            &(PROJECT.to_string() + "/" + &project_slug + MEMBERS),
        )
        .expect("request_api"),
    )
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
        project
            .loaders
            .iter()
            .map(|e| e.to_string() + ",")
            .collect::<String>(),
        project
            .game_versions
            .iter()
            .rev()
            .take(10)
            .map(|e| "  ".to_string() + &e.to_string() + "\n")
            .collect::<String>(),
        project.license.name,
        match project.source_url {
            Some(v) => {
                v.bright_blue()
            }
            None => {
                "none".to_string().red()
            }
        },
        members
            .iter()
            .map(|mem| "  ".to_string() + &mem.user.username.clone() + ", " + &mem.role + "\n")
            .collect::<String>(),
    );
}

#[deprecated]
pub fn get_project_version(
    client: &Client,
    staging: usize,
    project_slug: String,
    version_desc: MVDescriptor,
) -> Result<Version, ApiError> {
    let mut project_version: Option<Version> = None;
    let versions: Vec<Version> = match serde_json::from_value(
        match request_api(
            client,
            staging,
            &(PROJECT.to_owned() + "/" + &project_slug + VERSION),
        ) {
            Ok(v) => v,
            Err(e) => {
                println!("{}", e.to_string());
                return Err(ApiError::not_found());
            }
        },
    ) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e.to_string());
            return Err(ApiError::invalid_data());
        }
    };
    if version_desc.mc_ver.is_latest() {
        project_version = Some(versions[0].clone());
    } else {
        for version in versions {
            if version_desc.check_version_compat(&version) {
                project_version = Some(version.clone());
                break;
            }
        }
    }

    if project_version.is_none() {
        return Err(ApiError::not_found());
    }

    Ok(project_version.expect("Unknown Error"))
}

#[deprecated]
pub fn get_project(
    client: &Client,
    staging: usize,
    project_slug: String,
) -> Result<Project, String> {
    let project: Project = serde_json::from_value(
        request_api(
            client,
            staging,
            &(PROJECT.to_string() + "/" + &project_slug),
        )
        .expect("request_api"),
    )
    .expect("from_value");

    Ok(project)
}
