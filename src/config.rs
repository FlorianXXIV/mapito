use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{create_dir_all, File},
    io::{ErrorKind, Read, Write},
    str::FromStr,
};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub enum VT {
    RELEASE,
    BETA,
    ALPHA,
}

impl VT {
    pub fn to_string(&self) -> String {
        match self {
            Self::RELEASE => String::from_str("release").expect("from_str"),
            Self::BETA => String::from_str("beta").expect("from_str"),
            Self::ALPHA => String::from_str("alpha").expect("from_str"),
        }
    }
}

impl FromStr for VT {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "release" => Ok(Self::RELEASE),
            "RELEASE" => Ok(Self::RELEASE),
            "beta" => Ok(Self::BETA),
            "BETA" => Ok(Self::BETA),
            "alpha" => Ok(Self::ALPHA),
            "ALPHA" => Ok(Self::ALPHA),
            _ => Err("invalid version type".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Configuration {
    release_type: VT,
    download_path: String,
    pack_path: String,
}

pub fn configure() -> Result<(VT, String, String), String> {
    let config: Configuration;

    let config_path = env::var("HOME").unwrap() + "/.config/modrinth-apitool";
    let mut config_fd = match File::open(config_path + "/config.toml") {
        Ok(v) => v,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => create_config().expect("create_config"),
            ek => return Err(ek.to_string()),
        },
    };

    let mut body = String::new();
    config_fd.read_to_string(&mut body).expect("read_to_string");

    config = toml::from_str(body.as_str()).expect("toml::from_str");

    Ok((config.release_type, config.download_path, config.pack_path))
}

fn create_config() -> Result<File, std::io::Error> {
    create_dir_all(env::var("HOME").unwrap() + "/.config/modrinth-apitool")?;
    let mut config =
        File::create(env::var("HOME").unwrap() + "/.config/modrinth-apitool/config.toml")?;
    let defaults = Configuration {
        release_type: VT::RELEASE,
        download_path: env::var("HOME").unwrap() + "/Downloads",
        pack_path: env::var("HOME").unwrap() + "/.config/modrinth-apitool/packs",
    };
    write!(&mut config, "{}", toml::to_string(&defaults).unwrap())?;

    return Ok(config);
}
