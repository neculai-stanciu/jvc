mod archive;
mod client;
mod commands;
mod config;
mod loglevel;
mod provider;
mod shell;
mod version;

use std::time::Instant;

use anyhow::Result;
use commands::executor::Executor;
use config::JvcConfig;
use indicatif::HumanDuration;
use log::{debug, warn};
use loglevel::LogLevel;
use std::io::Write;
use structopt::StructOpt;

/// A cli application to handle all java version for windows, linux and macos
#[derive(Debug, StructOpt)]
#[structopt(name = "jvc")]
pub struct Cli {
    #[structopt(flatten)]
    pub config: JvcConfig,
    #[structopt(subcommand)]
    pub main_commands: SubCommand,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// List all available Java feature versions
    #[structopt(name="list", visible_aliases= &["ls"])]
    List(commands::list::List),
    /// List all available Java feature versions
    #[structopt(name="install", visible_aliases= &["i"])]
    Install(commands::install::Install),
    /// Remove an installed Java feature version
    #[structopt(name="remove", visible_aliases= &["rm"])]
    Remove(commands::remove::Remove),
    /// Support for other packages like maven, gradle based on configuration file
    #[structopt(name="package", visible_aliases= &["pkg"])]
    Package(commands::package::Package),
    /// Used for environment setup. For windows please use setup command
    Env(commands::env::Env),
    /// Set an alias for a java version
    Alias(commands::alias::Alias),
    /// Sets default java version
    Default(commands::default::Default),
    #[cfg(target_os = "windows")]
    /// One time setup for windows env
    Setup(commands::windows::Setup),
}

impl SubCommand {
    pub async fn execute(self, config: JvcConfig) -> Result<()> {
        debug!("Executing subcommand:");
        match self {
            SubCommand::List(executor) => executor.execute(config).await,
            SubCommand::Install(executor) => executor.execute(config).await,
            SubCommand::Env(executor) => executor.execute(config).await,
            SubCommand::Remove(executor) => executor.execute(config).await,
            SubCommand::Package(executor) => executor.execute(config).await,
            SubCommand::Alias(executor) => executor.execute(config).await,
            SubCommand::Default(executor) => executor.execute(config).await,
            #[cfg(target_os = "windows")]
            SubCommand::Setup(executor) => executor.execute(config).await,
        }
    }
}

fn init_logging(log_level: &LogLevel) {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(Some("jvc"), log_level.into())
        .init();
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let start_time = Instant::now();

    let args: Cli = Cli::from_args();
    let config = args.config;

    let result = config.clean_up_downloads_dir();
    if let Err(e) = result {
        warn!("Cannot clean up downloads dir: {}", e)
    }

    init_logging(&config.log_level);
    args.main_commands.execute(config).await?;

    let end = start_time.elapsed();
    debug!("Time elapsed is: {}", HumanDuration(end));
    Ok(())
}
