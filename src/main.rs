mod config;
mod module;

use clap::Parser;
use eyre::Context;
use std::{fs::read_to_string, path::PathBuf, process::exit};

use crate::config::Config;

#[derive(Debug, Parser)]
#[command(version, about)]
struct App {
    #[clap(default_value = "splice.toml")]
    config_path: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

impl App {
    fn config(&self) -> eyre::Result<Config> {
        let contents = read_to_string(&self.config_path).wrap_err_with(|| {
            format!(
                "could not read config file at `{}`",
                self.config_path.display()
            )
        })?;

        toml::from_str(&contents).wrap_err("could not parse config file as TOML")
    }

    fn run(self) -> eyre::Result<()> {
        let config = self.config()?;
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
            Self::Sync => {
                println!("TODO: sync!");
                Ok(())
            }
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
