mod config;
mod module;
mod sync;
mod validator;

use clap::Parser;
use eyre::Context;
use std::{path::PathBuf, process::exit};
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, filter::LevelFilter, fmt};

use crate::config::Config;

#[derive(Debug, Parser)]
#[command(version, about)]
struct App {
    // Path to the TOML config file
    #[clap(long, short, default_value = config::FILE_NAME)]
    config_path: PathBuf,

    #[clap(long, env)]
    no_color: bool,

    /// The lowest severity of log to print
    #[clap(long, env, default_value_t = Level::INFO)]
    log_level: Level,

    #[command(subcommand)]
    command: Option<Command>,
}

impl App {
    fn run(self) -> eyre::Result<()> {
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_target(false)
                    .with_ansi(!self.no_color)
                    .without_time()
                    .with_filter(LevelFilter::from_level(self.log_level)),
            )
            .with(ErrorLayer::default())
            .init();

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
    let app = App::parse();

    let mut hooks = color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false);
    if app.no_color {
        // A blank theme strips the ANSI styling from the error report, keeping
        // it consistent with the logger's `--no-color` behavior.
        hooks = hooks.theme(color_eyre::config::Theme::new());
    }
    hooks
        .install()
        .expect("failed to install the color_eyre error handler");

    if let Err(err) = app.run() {
        eprintln!("{err:?}");
        exit(1);
    }
}
