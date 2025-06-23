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

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
}
