use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use env_logger::Env;

use ffi_log2::log_param;
use hamsrs::hams_logger_init;

use log::{debug, error, info};
use log4ham::config::MyConfig;
use log4ham::{
    error::MyError,
    persistence::{start_db_backup, start_db_check_tables, start_db_migrate},
    webserver::service_start,
};

use log4ham::{NAME, VERSION};

/// Application definition to defer to set of commands under [Commands]
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

/// Commands to run inside this program
#[derive(Debug, Subcommand)]
enum Commands {
    /// Show version of application
    Version,
    /// Migrate the sql actions to the DB
    Migrate {
        /// Sets a custom config file
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
        /// Sets a custom secrets directory
        #[arg(short, long, value_name = "DIR", default_value = PathBuf::from("secrets").into_os_string())]
        secrets: PathBuf,
    },
    /// Start the http service
    Start {
        /// Sets a custom config file
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
        /// Sets a custom secrets directory
        #[arg(short, long, value_name = "DIR", default_value = PathBuf::from("secrets").into_os_string())]
        secrets: PathBuf,
    },
    /// DB Check
    DbCheck {
        /// Sets a custom config file
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
        /// Sets a custom secrets directory
        #[arg(short, long, value_name = "DIR", default_value = PathBuf::from("secrets").into_os_string())]
        secrets: PathBuf,
    },
    /// DB Backup
    Backup {
        /// Sets a custom config file
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
        /// Sets a custom secrets directory
        #[arg(short, long, value_name = "DIR", default_value = PathBuf::from("secrets").into_os_string())]
        secrets: PathBuf,

        /// define the backup directory
        #[arg(value_name = "BACKUPDIR")]
        backup_dir: PathBuf,
    },
    ConfigCheck {
        /// Sets a custom config file
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
        /// Sets a custom secrets directory
        #[arg(short, long, value_name = "DIR", default_value = PathBuf::from("secrets").into_os_string())]
        secrets: PathBuf,
    },
}

fn main() -> Result<ExitCode, MyError> {
    let log_level = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(log_level).init();

    let args = Args::parse();
    match args.command {
        Commands::Version => {
            info!("Version: {NAME}:{VERSION}");
            info!("HaMs Version: {}", hamsrs::hams_version());
        }
        Commands::Start { config, secrets } => {
            info!("Starting {NAME}:{VERSION}");
            info!("Starting {}", hamsrs::hams_version());

            hams_logger_init(log_param()).unwrap();

            let config_yaml = std::fs::read_to_string(config.clone())?;

            let config: MyConfig = MyConfig::figment(&config_yaml, secrets)
                .extract()
                .unwrap_or_else(|err| {
                    error!("Config file {config:?} failed with error \n{err:#?}");
                    panic!("Config failed to load");
                });

            debug!("Loaded config {:?}", config);

            if config.persistence.db.automigrate {
                info!("Auto-migrating database");
                start_db_migrate(&config.persistence)?;
            }

            service_start(&config)?;
        }
        Commands::DbCheck { config, secrets } => {
            info!("Starting {NAME} for {VERSION}");

            let config_yaml = std::fs::read_to_string(config.clone())?;

            let config: MyConfig = MyConfig::figment(&config_yaml, secrets).extract()?;

            debug!("Loaded config {:#?}", config);

            start_db_check_tables(&config.persistence)?;
        }
        Commands::ConfigCheck { config, secrets } => {
            info!("Config check {NAME} for {VERSION}");

            let config_yaml = std::fs::read_to_string(config.clone())?;

            let config: MyConfig = MyConfig::figment(&config_yaml, secrets).extract()?;

            debug!("Loaded config {:#?}", config);
        }
        Commands::Migrate { config, secrets } => {
            info!("Starting Migration for {NAME}:{VERSION}");

            let config_yaml = std::fs::read_to_string(config.clone())?;

            let config: MyConfig = MyConfig::figment(&config_yaml, secrets).extract()?;

            debug!("Loaded config {:#?}", config);

            start_db_migrate(&config.persistence)?;
        }
        Commands::Backup {
            config,
            secrets,
            backup_dir,
        } => {
            info!("Starting DB Backup for {NAME}:{VERSION}");

            let config_yaml = std::fs::read_to_string(config.clone())?;

            let config: MyConfig = MyConfig::figment(&config_yaml, secrets).extract()?;

            debug!("Loaded config {:#?}", config);

            start_db_backup(&config.persistence, &backup_dir)?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
