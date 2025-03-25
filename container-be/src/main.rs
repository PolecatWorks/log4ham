use std::path::PathBuf;

use clap::{Parser, Subcommand};
use env_logger::Env;

use log::{debug, error, info};
use log4ham::{
    app_schema::{schema_person_string, schema_string, write_records, Person},
    error::MyError,
    persistence::db_check,
    webserver::service_start,
};

use log4ham::{webserver::MyConfig, NAME, VERSION};

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
    /// Generate messages
    Generate {
        /// filename to write content to
        filename: String,

        /// Number of records
        #[arg(long, default_value_t = 1)]
        count: u32,
    },
    // TODO: Add subcommand to generate our API objects with parameters
    /// Show schema for object
    Schema,
    /// Show schema for Vec of object
    SchemaList,
    /// Validate file against schema
    Validate {
        /// filename to read content from
        filename: String,
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
    ConfigCheck {
        /// Sets a custom config file
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
        /// Sets a custom secrets directory
        #[arg(short, long, value_name = "DIR", default_value = PathBuf::from("secrets").into_os_string())]
        secrets: PathBuf,
    },
}

fn main() -> Result<(), MyError> {
    let log_level = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(log_level).init();

    let args = Args::parse();
    match args.command {
        Commands::Generate { filename, count } => {
            println!("Creating filename {filename} and writing {count} records");
            write_records(&filename, count)?;
        }
        Commands::Schema => println!("{}", schema_person_string()?),
        Commands::SchemaList => println!("{}", schema_string::<Vec<Person>>()?),
        Commands::Validate { filename } => todo!("Validate {filename}"),
        Commands::Start { config, secrets } => {
            info!("Starting {NAME} at {VERSION}");

            let config_yaml = std::fs::read_to_string(config.clone())?;

            let config: MyConfig = MyConfig::figment(&config_yaml, secrets)
                .extract()
                .unwrap_or_else(|err| {
                    error!("Config file {config:?} failed with error \n{err:#?}");
                    panic!("Config failed to load");
                });

            debug!("Loaded config {:?}", config);

            service_start(config)?
        }
        Commands::DbCheck { config, secrets } => {
            info!("Starting {NAME} for {VERSION}");

            let config_yaml = std::fs::read_to_string(config.clone())?;

            let config: MyConfig = MyConfig::figment(&config_yaml, secrets).extract()?;

            info!("Loaded config {:#?}", config);

            db_check(config.persistence)?;
        }
        Commands::ConfigCheck { config, secrets } => {
            info!("Config check {NAME} for {VERSION}");

            let config_yaml = std::fs::read_to_string(config.clone())?;

            let config: MyConfig = MyConfig::figment(&config_yaml, secrets).extract()?;

            info!("Loaded config {:#?}", config);
        }
    }

    Ok(())
}
