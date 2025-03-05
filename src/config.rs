use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{create_dir_all, File},
    io::{ErrorKind, Read, Write},
    str::FromStr,
};
use toml::{self, Table};

use crate::mrapi::defines::{LOADER, VT};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    pub release_type: VT,
    pub loader: LOADER,
    pub download_path: String,
    pub pack_path: String,
    pub mc_ver: String,
    pub staging: usize,
}

pub fn configure() -> Result<Configuration, String> {
    let config: Configuration;

    let config_path = env::var("HOME").unwrap() + "/.config/modrinth-apitool";
    let mut config_fd = match File::open(config_path.clone() + "/config.toml") {
        Ok(v) => v,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => create_config().expect("create_config"),
            ek => return Err(ek.to_string()),
        },
    };

    let mut body = String::new();
    config_fd.read_to_string(&mut body).expect("read_to_string");

    config = parse_config(body)?;

    let mut config_fd = File::create(config_path + "/config.toml").expect("open");

    write!(&mut config_fd, "{}", toml::to_string(&config).unwrap()).expect("write config");
    Ok(config)
}

fn create_config() -> Result<File, std::io::Error> {
    create_dir_all(env::var("HOME").unwrap() + "/.config/modrinth-apitool")?;
    let mut config =
        File::create(env::var("HOME").unwrap() + "/.config/modrinth-apitool/config.toml")?;
    let defaults = get_default_cfg();
    write!(&mut config, "{}", toml::to_string(&defaults).unwrap())?;

    return Ok(config);
}

fn parse_config(body: String) -> Result<Configuration, String> {
    let mut config = get_default_cfg();
    let cfg_table = match body.parse::<Table>() {
        Ok(v) => v,
        Err(e) => return Err(e.message().to_string()),
    };

    for (key, value) in cfg_table {
        match key.as_str() {
            "release_type" => config.release_type = VT::from_str(value.as_str().unwrap()).unwrap(),
            "loader" => config.loader = LOADER::from_str(value.as_str().unwrap()).unwrap(),
            "download_path" => config.download_path = value.try_into().unwrap(),
            "pack_path" => config.pack_path = value.try_into().unwrap(),
            "mc_ver" => config.mc_ver = value.try_into().unwrap(),
            "staging" => config.staging = value.try_into().unwrap(),
            &_ => println!("Warning: unused key '{key}' in config file."),
        }
    }

    Ok(config)
}

fn get_default_cfg() -> Configuration {
    Configuration {
        release_type: VT::RELEASE,
        download_path: env::var("HOME").unwrap() + "/Downloads",
        pack_path: env::var("HOME").unwrap() + "/.config/modrinth-apitool/packs",
        loader: LOADER::FABRIC,
        mc_ver: "latest".to_string(),
        staging: 0,
    }
}
