use std::borrow::Borrow;

use colored::Colorize;
use reqwest::{blocking::Client, blocking::Response, Url};
use serde_json::Value;

use crate::{
    mc_info::MCVersion,
    mrapi::{constants::MEMBERS, defines::Member},
    util::error::ApiError,
};

use super::{
    constants::{API_URL, FACETS, LIMIT, OFFSET, PROJECT, QUERY, SEARCH},
    defines::{Project, SearchResp},
};

#[derive(Debug)]
pub struct ApiClient {
    client: Client,
    staging: usize,
}

impl ApiClient {
    /// create a new api client, that can send requests to either modrinths normal endpoint
    /// or to the staging server depending on what bit is set.
    pub fn new(staging: usize) -> ApiClient {
        ApiClient {
            client: Client::new(),
            staging: staging,
        }
    }

    /// send a single request to modrinths api, with the given endpoint
    fn request_api_par<I, K, V>(&self, endpoint: &String, params: I) -> Result<Response, ApiError>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let query =
            Url::parse_with_params(&(API_URL[self.staging].to_owned() + endpoint), params).unwrap();
        Ok(self.client.get(query).send()?)
    }

    fn request_api(&self, endpoint: &String) -> Result<Response, ApiError> {
        let query = Url::parse(&(API_URL[self.staging].to_owned() + endpoint)).unwrap();
        Ok(self.client.get(query).send()?)
    }

    pub fn search(
        &self,
        query: &String,
        limit: Option<usize>,
        offset: Option<usize>,
        facets: &Option<Vec<Vec<(String, String)>>>,
    ) -> Result<Vec<String>, ApiError> {
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
                let mut str_facet: String = "[".to_string();
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
                self.request_api_par(
                    &SEARCH.to_string(),
                    &[
                        (QUERY, query),
                        (LIMIT, &par_limit),
                        (OFFSET, &par_offset),
                        (FACETS, &str_facet),
                    ],
                )
            }
            None => self.request_api_par(
                &SEARCH.to_string(),
                &[(QUERY, query), (LIMIT, &par_limit), (OFFSET, &par_offset)],
            ),
        }?;
        let query_response: SearchResp = query.json()?;

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

        Ok(slugs)
    }

    pub fn get_project(&self, project_slug: &String) -> Result<Project, String> {
        let project: Project = self
            .request_api(&(PROJECT.to_string() + "/" + project_slug))
            .expect("request_api")
            .json()
            .expect("json");

        Ok(project)
    }

    pub fn print_project_info(&self, project_slug: &String) {
        let project = self.get_project(project_slug).expect("get_project");
        let members: Vec<Member> = self
            .request_api(&(PROJECT.to_string() + "/" + project_slug + MEMBERS))
            .expect("request_api")
            .json()
            .expect("json");
        println!("{}", project);
        println!(
            "members:\n{}",
            members
                .iter()
                .map(|m| { m.to_string() + "\n" })
                .collect::<String>()
        )
    }

}
