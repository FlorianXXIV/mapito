use std::fmt::Display;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::mc_info::MCVersion;
use crate::mc_info::LOADER;
use crate::mc_info::VT;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResp {
    pub hits: Vec<Value>,
    offset: i32,
    limit: i32,
    total_hits: i32,
}

//A specific version of a project

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub game_versions: Vec<MCVersion>,
    pub loaders: Vec<LOADER>,
    pub name: String,
    pub version_number: String,
    pub downloads: u32,
    pub version_type: VT,
    pub files: Vec<ApiFile>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiFile {
    pub url: String,
    pub hashes: Map<String, Value>,
    pub filename: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Dependency {
    pub project_id: String,
    pub dependency_type: String,
}

/// A modrinth Project, this can be a mod, modpack, resourcepack or shader
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub slug: String,
    pub project_type: String,
    pub team: String,
    pub title: String,
    pub description: String,
    pub published: String,
    pub updated: String,
    pub license: License,
    pub downloads: u32,
    pub game_versions: Vec<MCVersion>,
    pub categories: Vec<String>,
    pub loaders: Vec<LOADER>,
    pub source_url: Option<String>,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Project: {}, latest-{}, {}\n {}\n\n Released: {}\n Last Updated: {} \n \
        loaders: {}\n supported versions: \n{} license: {}\n source: {}\n",
            self.title,
            self.game_versions.last().expect("last"),
            self.project_type.green(),
            self.description,
            self.published.yellow(),
            self.updated.yellow(),
            self.loaders
                .iter()
                .map(|e| e.to_string() + ",")
                .collect::<String>(),
            self.game_versions
                .iter()
                .rev()
                .take(10)
                .map(|e| "  ".to_string() + &e.to_string() + "\n")
                .collect::<String>(),
            self.license.name,
            match &self.source_url {
                Some(v) => {
                    v.bright_blue()
                }
                None => {
                    "none".to_string().red()
                }
            },
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

/// The Members of a Team
#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub role: String,
    pub team_id: String,
    pub user: User,
}

impl Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.role, self.user.username)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
}
