mod env;
mod ignore;
mod volume;

use std::{
    env::current_dir, os::unix::process::CommandExt, path::PathBuf, process::Command, str::FromStr,
};

use anyhow::{Result, anyhow, ensure};
use log::info;
use serde::de::IntoDeserializer;

use crate::{
    config::Config,
    container::{ignore::parse_ignore_rule_set, volume::volume_mappings_by_ignore_rule_sets},
};

pub struct Container<'a> {
    config: &'a Config,
    debug: bool,
}

impl<'a> Container<'a> {
    pub fn new(config: &'a Config, debug: bool) -> Self {
        Self { config, debug }
    }

    pub fn build(&self) -> Result<()> {
        info!("Building Dockerfile...");
        let build_command = Command::new("docker")
            .arg("build")
            .args([
                "-f",
                format!("{}", self.config.dockerfile_path().display()).as_str(),
                "-t",
                self.image_name().as_str(),
                ".",
            ])
            .output()
            .expect("Failed to run docker build");

        if self.debug {
            String::from_utf8_lossy(&build_command.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .for_each(|l| info!("{}", l));
            String::from_utf8_lossy(&build_command.stderr)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .for_each(|l| info!("{}", l));
        }
        info!("Built Dockerfile");
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        info!("Starting Claudestine...");
        let mappings = volume_mappings_by_ignore_rule_sets(
            self.config
                .ignore_files
                .iter()
                .chain(
                    Some(
                        &Config::default_isolates_path()
                            .to_str()
                            .unwrap()
                            .to_string(),
                    )
                    .into_iter(),
                )
                .map(|i| parse_ignore_rule_set(PathBuf::from_str(i).unwrap()))
                .collect(),
        );
        let mut command_builder = Command::new("docker");
        command_builder
            .arg("run")
            .arg("--rm")
            .arg("--interactive")
            .arg("--tty")
            .arg("--env")
            .arg("TERM=xterm-256color");
        for rule in mappings {
            let arg = rule.serialize();
            command_builder.arg("--volume").arg(arg.as_str());
        }
        command_builder.arg(self.image_name());
        let cmd_str = std::iter::once(command_builder.get_program())
            .chain(command_builder.get_args())
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        if self.debug {
            info!("Running: {}", cmd_str);
        }
        let _ = command_builder.exec();
        info!("Stopping Claudestine");
        Ok(())
    }

    fn image_name(&self) -> String {
        format!(
            "{}-claudestine:{}",
            &self.config.workspace_identifier,
            env!("CARGO_PKG_VERSION")
        )
    }
}
