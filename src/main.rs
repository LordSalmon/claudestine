use std::{
    env::current_dir,
    fs::{self, create_dir, remove_dir_all},
    process::ExitStatus,
};

use log::{error, info};
mod config;
mod container;
mod setup;

use clap::{Parser, Subcommand};

use crate::{config::Config, container::Container};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Run {
        #[arg(long, default_value_t = false)]
        debug: bool,
    },
    Init {
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    Setup {},
    Build {
        #[arg(long, default_value_t = false)]
        debug: bool,
    },
    Cleanup {},
    Update {},
}

#[derive(Subcommand)]
enum ConfigCommand {
    Show {},
}

#[derive(Subcommand)]
enum RunCommand {}

fn main() {
    simple_logger::init_utc().unwrap();
    let cli = Cli::parse();

    match cli.command {
        Command::Config { command } => match command {
            ConfigCommand::Show {} => match Config::init() {
                Ok(cfg) => cfg.pretty_print(),
                Err(e) => error!("{}", e),
            },
        },
        Command::Run { debug } => {
            if let Ok(config) = Config::init() {
                info!("Welcome to claudestine!");
                let container = Container::new(&config, debug);
                if let Ok(()) = container.build() {
                    container.start().unwrap()
                } else {
                    error!("Couldn't build the container. Try running it with --debug");
                }
            } else {
                error!("Couldn't initialize the configuration");
            }
        }
        Command::Init { force } => {
            if Config::config_directory().exists() {
                if !force {
                    info!("Claudestine is already initialized. Skipping...");
                    return;
                } else {
                    remove_dir_all(Config::config_directory()).unwrap();
                    create_dir(Config::config_directory()).unwrap();
                }
            } else {
                create_dir(Config::config_directory()).unwrap();
            }
            if let Some(project_identifier) = current_dir().unwrap().file_name() {
                fs::write(
                    Config::config_file_path(),
                    format!(
                        "workspace_identifier = \"{}\"\nignore_files = []",
                        project_identifier.to_str().unwrap()
                    ),
                )
                .map_err(|_| "Couldn't write to config file.")
                .unwrap();
                fs::write(
                    Config::default_dockerfile_path(),
                    include_bytes!("../assets/Dockerfile"),
                )
                .map_err(|_| "Couldn't write to Dockerfile.")
                .unwrap();
                fs::write(Config::default_isolates_path(), "")
                    .map_err(|_| "Couldn't create the isolates file.")
                    .unwrap();
                info!("Claudestine successfully initialized.")
            } else {
                error!("Couldn't read the current directories name");
            }
        }
        Command::Build { debug } => {
            if let Ok(config) = Config::init() {
                let container = Container::new(&config, debug);
                if let Err(e) = container.build() {
                    error!("Couldn't build the container: {}", e);
                }
            } else {
                error!("Couldn't initialize the configuration");
            }
        }
        Command::Setup {} => {
            info!("Setting up Claudestine...");
            if let Err(e) = setup::setup() {
                error!("Couldn't setup Claudestine");
                error!("{:?}", e);
            } else {
                info!("Claudestine is set up!");
            }
        }
        Command::Update {} => {
            info!("Updating Claudestine...");
            let install_script =
                "https://raw.githubusercontent.com/LordSalmon/claudestine/main/scripts/install.sh";
            let status = std::process::Command::new("sh")
                .args(["-c", &format!("curl -fsSL '{}' | sh", install_script)])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .expect("Failed to run update script");
            if status.success() {
                info!("Claudestine is now updated!");
            } else {
                error!("Update failed");
            }
        }
        Command::Cleanup {} => {
            let version = env!("CARGO_PKG_VERSION");
            let in_use: std::collections::HashSet<String> = {
                let out = std::process::Command::new("docker")
                    .args(["ps", "--format", "{{.Image}}"])
                    .output()
                    .expect("Failed to run docker ps");
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .map(str::to_owned)
                    .collect()
            };
            let output = std::process::Command::new("docker")
                .args(["images", "--format", "{{.Repository}}:{{.Tag}}"])
                .output()
                .expect("Failed to run docker images");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let to_delete: Vec<&str> = stdout
                .lines()
                .filter(|line| {
                    line.ends_with(&format!("-claudestine:{}", version)) && !in_use.contains(*line)
                })
                .collect();
            if to_delete.is_empty() {
                info!("No claudestine images to clean up.");
            } else {
                for image in &to_delete {
                    info!("Removing image: {}", image);
                }
                let status = std::process::Command::new("docker")
                    .arg("rmi")
                    .args(&to_delete)
                    .status()
                    .expect("Failed to run docker rmi");
                if !status.success() {
                    error!("docker rmi failed");
                }
            }
        }
    }
}
