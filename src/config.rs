use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Display,
    fs::{create_dir_all, File},
    io::{ErrorKind, Read, Write},
    path::PathBuf,
    str::FromStr,
};
use toml::{self, Table};

use crate::mc_info::{Loader, MCVersion, MCVersionUtils, VT};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    pub release_type: VT,
    pub loader: Loader,
    pub download_path: String,
    pub pack_path: String,
    pub mc_ver: MCVersion,
    pub staging: usize,
    pub install_path: Option<String>,
}

impl Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Release Type: {}\nLoader: {}\nDownload Path: {}\nPack Path: {}\nMinecraft Version: {}\nStaging: {}\nInstallation Path: {}",
            self.release_type,
            self.loader,
            self.download_path,
            self.pack_path,
            self.mc_ver,
            self.staging,
            self.install_path.clone().unwrap_or("none".to_string())
        )
    }
}

pub fn configure() -> Result<Configuration, String> {
    let config_dir = config_path()?;

    let mut config_fd = match File::open(config_dir.as_path()) {
        Ok(v) => v,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => create_config().expect("create_config"),
            ek => return Err(ek.to_string()),
        },
    };

    let mut body = String::new();
    config_fd.read_to_string(&mut body).expect("read_to_string");

    let config: Configuration = parse_config(body)?;

    let mut config_fd = File::create(config_dir.as_path()).expect("open");

    write!(&mut config_fd, "{}", toml::to_string(&config).unwrap()).expect("write config");

    Ok(config)
}

fn create_config() -> Result<File, std::io::Error> {
    let config_dir = match env::home_dir() {
        Some(path) => path.join(".config/mapito"),
        None => {
            return Err(std::io::Error::last_os_error());
        }
    };
    create_dir_all(config_dir.as_path())?;
    let mut config = File::create(config_dir.join("config.toml"))?;
    let defaults = get_default_cfg();
    write!(&mut config, "{}", toml::to_string(&defaults).unwrap())?;

    Ok(config)
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
            "loader" => config.loader = Loader::from_str(value.as_str().unwrap()).unwrap(),
            "download_path" => config.download_path = value.try_into().unwrap(),
            "pack_path" => config.pack_path = value.try_into().unwrap(),
            "mc_ver" => config.mc_ver = value.try_into().unwrap(),
            "staging" => config.staging = value.try_into().unwrap(),
            "install_path" => config.install_path = Some(value.try_into().unwrap()),
            &_ => println!("Warning: unused key '{key}' in config file."),
        }
    }

    Ok(config)
}

fn get_default_cfg() -> Configuration {
    Configuration {
        release_type: VT::Release,
        download_path: env::home_dir()
            .unwrap()
            .join("Downloads")
            .to_str()
            .unwrap()
            .to_owned(),
        pack_path: env::home_dir()
            .unwrap()
            .join(".config/mapito/packs")
            .to_str()
            .unwrap()
            .to_owned(),
        loader: Loader::Fabric,
        mc_ver: MCVersion::latest(),
        staging: 0,
        install_path: None,
    }
}

pub fn config_path() -> Result<PathBuf, String> {
    match env::home_dir() {
        Some(path) => Ok(path.join(".config/mapito/config.toml")),
        None => Err("Home Dir not Found".to_owned()),
    }
}
