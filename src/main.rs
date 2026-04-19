use log::{error, info};
mod config;
mod container;

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
                container.build();
                container.start().unwrap();
            } else {
                error!("Couldn't initialize the configuration");
            }
        }
    }
}
