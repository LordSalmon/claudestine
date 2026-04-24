pub mod env;
mod ignore;
mod volume;

use std::{
    env::temp_dir,
    fs,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};

use anyhow::Result;
use log::info;

use crate::{
    broker::BrokerServer,
    config::Config,
    container::{
        env::EnvRecord,
        ignore::parse_ignore_rule_set,
        volume::{VolumeMapping, claudestine_config_mapping, volume_mappings_by_ignore_rule_sets},
    },
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
        info!("Building Dockerfile. This may take a while...");
        let mut command = Command::new("docker");
        command.args([
            "build",
            "-f",
            format!("{}", self.config.dockerfile_path().display()).as_str(),
            "-t",
            self.image_name().as_str(),
            ".",
        ]);
        if self.debug {
            command.status().expect("Failed to run docker build");
        } else {
            command.output()?;
        }

        info!("Built Dockerfile");
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        let environment_records: [Option<EnvRecord>; 0] = [];
        info!("Starting Claudestine...");
        let mut mappings = volume_mappings_by_ignore_rule_sets(
            self.config
                .ignore_files()
                .iter()
                .map(|i| parse_ignore_rule_set(PathBuf::from_str(i).unwrap()))
                .collect(),
        );
        mappings.push(claudestine_config_mapping());

        // Set up broker if configured and enabled
        let broker_setup = self.setup_broker(&mut mappings)?;

        let mut command_builder = Command::new("docker");
        command_builder
            .arg("run")
            .arg("--rm")
            .arg("--interactive")
            .arg("--tty")
            .arg("--env")
            .arg("TERM=xterm-256color");

        for environment_mapping in environment_records.iter().flatten() {
            command_builder
                .arg("--env")
                .arg(environment_mapping.serialize());
        }

        if let Some((port, _)) = &broker_setup {
            command_builder
                .arg("--add-host")
                .arg("host.docker.internal:host-gateway")
                .arg("--env")
                .arg(format!("BROKER_PORT={port}"));
        }

        for rule in mappings {
            command_builder.arg("--volume").arg(rule.serialize());
        }
        command_builder.arg(self.image_name());

        if self.debug {
            let cmd_str = std::iter::once(command_builder.get_program())
                .chain(command_builder.get_args())
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            info!("Running: {}", cmd_str);
        }

        if broker_setup.is_some() {
            // Keep parent process alive to serve broker requests
            command_builder
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
                .wait()?;

            if let Some((_, script_path)) = broker_setup {
                let _ = fs::remove_file(script_path);
            }
        } else {
            let _ = command_builder.exec();
        }

        info!("Stopping Claudestine");
        Ok(())
    }

    /// If broker is enabled, writes the client script to a temp path, starts the TCP
    /// server, and appends the script volume mount to `mappings`.
    /// Returns (port, temp_script_path) on success, or None if broker is not active.
    fn setup_broker(&self, mappings: &mut Vec<VolumeMapping>) -> Result<Option<(u16, PathBuf)>> {
        let Some(broker_cfg) = &self.config.broker else {
            return Ok(None);
        };
        if !broker_cfg.enabled || broker_cfg.commands.is_empty() {
            return Ok(None);
        }

        info!("Starting broker...");

        // Write the client script to a stable temp path per workspace
        let script_path = temp_dir().join(format!(
            "claudestine-{}-broker",
            self.config.workspace_identifier
        ));
        fs::write(&script_path, crate::broker::CLIENT_SCRIPT)?;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;

        mappings.push(VolumeMapping::new(
            script_path.clone(),
            PathBuf::from("/usr/local/bin/broker"),
        ));

        // Start the TCP server; block until it's bound and ready
        let server = BrokerServer::new(broker_cfg.commands.clone());
        let port_rx = server.start()?;
        let port = port_rx
            .recv()
            .map_err(|_| anyhow::anyhow!("Broker server failed to start"))?;

        info!("Broker: ready on port {}", port);
        Ok(Some((port, script_path)))
    }

    fn image_name(&self) -> String {
        format!(
            "{}-claudestine:{}",
            &self.config.workspace_identifier,
            env!("CARGO_PKG_VERSION")
        )
    }
}
