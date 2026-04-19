use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Result, ensure};
use log::{debug, error, info};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub workspace_identifier: String,
    pub ignore_files: Vec<String>,
    dockerfile_path: Option<String>,
}

impl Config {
    pub fn pretty_print(&self) {
        debug!("{:#?}", self);
    }

    fn config_path() -> PathBuf {
        Path::new(".claudestine").to_path_buf()
    }

    fn config_file_path() -> PathBuf {
        Self::config_path().join("config.toml")
    }

    fn default_dockerfile_path() -> PathBuf {
        Self::config_path().join("Dockerfile")
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
        }
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
