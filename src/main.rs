mod config;
mod module;
mod sync;

use clap::Parser;
use eyre::Context;
use std::{path::PathBuf, process::exit};

use crate::config::Config;

#[derive(Debug, Parser)]
#[command(version, about)]
struct App {
    #[clap(default_value = config::FILE_NAME)]
    config_path: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

impl App {
    fn run(self) -> eyre::Result<()> {
        let config = config::read(&self.config_path)?;
        self.command.unwrap_or(Command::Sync).run(config)
    }
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Sync templates in this directory according to the config (the default command)
    Sync,

    /// Show+manipulate configuration
    Config {
        #[command(subcommand)]
        subcommand: ConfigCommand,
    },
}

impl Command {
    fn run(&self, config: Config) -> eyre::Result<()> {
        match self {
            Self::Sync => sync::sync(config.try_app()?),
            Self::Config { subcommand } => subcommand.run(config),
        }
    }
}

#[derive(Debug, clap::Subcommand)]
enum ConfigCommand {
    /// Show the full calculated config
    Show,
}

impl ConfigCommand {
    fn run(&self, config: Config) -> eyre::Result<()> {
        match self {
            Self::Show => Self::print_config(config),
        }
    }

    fn print_config(config: Config) -> eyre::Result<()> {
        let pretty =
            toml::to_string_pretty(&config).wrap_err("could not serialize command to string")?;

        println!("{pretty}");

        Ok(())
    }
}

fn main() {
    color_eyre::install().unwrap();
    let app = App::parse();

    if let Err(err) = app.run() {
        eprintln!("{err:?}");
        exit(1);
    }
}
