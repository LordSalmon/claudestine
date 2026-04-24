use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Result, ensure};
use log::{debug, error, info};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BrokerArgDef {
    pub name: String,
    #[serde(rename = "type", default = "default_arg_type")]
    pub type_name: String,
    #[serde(default)]
    pub required: bool,
    pub default: Option<String>,
}

fn default_arg_type() -> String {
    "string".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrokerCommandDef {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub args: Vec<BrokerArgDef>,
    pub invoke: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrokerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub commands: Vec<BrokerCommandDef>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub workspace_identifier: String,
    pub ignore_files: Vec<String>,
    dockerfile_path: Option<String>,
    pub broker: Option<BrokerConfig>,
}

impl Config {
    pub fn pretty_print(&self) {
        debug!("{:#?}", self);
    }

    pub fn config_file_path() -> PathBuf {
        Self::config_directory().join("config.toml")
    }

    pub fn config_directory() -> PathBuf {
        Path::new(".claudestine").to_path_buf()
    }

    pub fn default_dockerfile_path() -> PathBuf {
        Self::config_directory().join("Dockerfile")
    }

    pub fn default_isolates_path() -> PathBuf {
        Self::config_directory().join("isolates")
    }

    pub fn dockerfile_path(&self) -> PathBuf {
        if let Some(path) = &self.dockerfile_path {
            return PathBuf::from_str(path.as_str()).unwrap();
        }
        Self::default_dockerfile_path()
    }

    fn default() -> Self {
        let current_folder = current_dir()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        Self {
            workspace_identifier: current_folder,
            dockerfile_path: None,
            ignore_files: Vec::new(),
            broker: None,
        }
    }

    pub fn ignore_files(&self) -> Vec<String> {
        let mut ignore_files = self.ignore_files.clone();
        ignore_files.push(Self::default_isolates_path().to_str().unwrap().to_string());
        ignore_files
    }

    pub fn init() -> Result<Self> {
        let config = Self::get();
        ensure!(
            Self::default_dockerfile_path().exists() && config.dockerfile_path.is_none(),
            "No Dockerfile or Dockerfile path provided."
        );
        Ok(config)
    }

    fn get() -> Self {
        let current_dir = current_dir();
        if let Err(e) = current_dir {
            error!("Couldn't read current dir!: {}", e);
        } else {
            info!(
                "Running claudestine in: {}",
                current_dir.unwrap().to_str().unwrap()
            );
        }

        if !(Self::config_file_path().exists()) {
            error!(
                "No {} found. Using default configuration",
                Self::config_file_path().to_str().unwrap()
            );
            Config::default()
        } else {
            let toml_config: Config = toml::from_str(
                fs::read_to_string(Self::config_file_path())
                    .unwrap()
                    .as_str(),
            )
            .unwrap();
            toml_config
        }
    }
}
