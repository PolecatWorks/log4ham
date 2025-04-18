use std::path::Path;
use std::{path::PathBuf, sync::Arc};

use ::parquet::data_type::{ByteArrayType, Int64Type};
use ::parquet::file::properties::WriterProperties;
use ::parquet::file::writer::SerializedFileWriter;
use log::{debug, info, warn};
use parquet::generate_parquet_schema_from_table;
use serde::Deserialize;
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, Column, Executor, PgPool};
use tokio_util::sync::CancellationToken;
use url::Url;

mod parquet;

use crate::config::UrlWithUsernamePassword;
use crate::{error::MyError, tokio_tools::run_in_tokio};

#[derive(Deserialize, Debug, Clone)]
pub struct DbConfig {
    pub pool_size: u32,
    pub connection: UrlWithUsernamePassword,
    pub automigrate: bool,
}

impl Default for DbConfig {
    fn default() -> Self {
        DbConfig {
            pool_size: 5,
            connection: UrlWithUsernamePassword {
                url: Url::parse("postgres://localhost:5432").unwrap(),
                username: None,
                password: None,
            },
            automigrate: true,
        }
    }
}

impl DbConfig {
    pub fn connection(&self) -> Url {
        self.connection.clone().into()
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct PersistenceConfig {
    pub db: DbConfig,
}

#[derive(Debug, Clone)]
pub struct PersistenceState {
    config: PersistenceConfig,
    pub pool_pg: PgPool,
}

impl PersistenceState {
    pub async fn new(config: &PersistenceConfig) -> Result<PersistenceState, MyError> {
        let pool_pg = PgPoolOptions::new()
            .max_connections(config.db.pool_size)
            .connect(config.db.connection().as_str())
            .await?;

        Ok(PersistenceState {
            config: config.clone(),
            pool_pg,
        })
    }
}

pub async fn db_count_records(
    ct: CancellationToken,
    config: &PersistenceConfig,
) -> Result<(), MyError> {
    let state = PersistenceState::new(config).await?;

    let pool_pg = state.pool_pg.clone();

    let select_reply = sqlx::query("SELECT 1").fetch_one(&pool_pg).await?;

    // iterate over tables called users,logs and count the number of records in each

    for table in ["users", "logs"] {
        let count_reply = sqlx::query(&format!("SELECT COUNT(*) FROM {}", table))
            .fetch_one(&pool_pg)
            .await?;

        info!("{} count records: {:?}", table, count_reply);
    }

    ct.cancel();

    Ok(())
}

pub fn start_db_check_tables(config: &PersistenceConfig) -> Result<(), MyError> {
    let ct = CancellationToken::new();

    let runtime = crate::tokio_tools::ThreadRuntime {
        threads: 0,
        stack_size: 0,
    };

    run_in_tokio("db_check", &runtime, db_count_records(ct, config))
}

pub async fn db_migrate(ct: CancellationToken, config: &PersistenceConfig) -> Result<(), MyError> {
    let state = PersistenceState::new(config).await?;

    let pool = state.pool_pg.clone();

    let select_reply = sqlx::query("SELECT 1").fetch_one(&pool).await?;

    // Run migrations
    // sqlx::migrate!() macro finds the migrations folder relative to Cargo.toml
    // It embeds the migration files into the binary at compile time.
    sqlx::migrate!() // Path relative to Cargo.toml
        .run(&pool)
        .await?;

    ct.cancel();

    Ok(())
}

pub fn start_db_migrate(config: &PersistenceConfig) -> Result<(), MyError> {
    let ct = CancellationToken::new();

    let runtime = crate::tokio_tools::ThreadRuntime {
        threads: 0,
        stack_size: 0,
    };

    run_in_tokio("db_check", &runtime, db_migrate(ct, config))
}

/// Backup the database to a Parquet file
/// This function will create a Parquet file in the specified backup directory
/// with the name "backup.parquet"
/// The Parquet file will contain the data from the "users" table
/// The schema will be generated automatically from the table description
pub async fn db_backup(
    ct: CancellationToken,
    config: &PersistenceConfig,
    backup_dir: &Path,
) -> Result<(), MyError> {
    let state = PersistenceState::new(config).await?;

    let pool = state.pool_pg.clone();

    let query = "SELECT * FROM users";

    let users_describe = pool.describe(query).await?;

    debug!("Describe result: {:?}", users_describe);

    let users_response = sqlx::query("SELECT * FROM users").fetch_all(&pool).await?;

    let schema_auto = generate_parquet_schema_from_table(&pool, "users").await?;
    debug!("Automatic Schema: {:#?}", schema_auto);

    let schema = schema_auto;

    let file_name = "backup.parquet";

    let abs_file_name = backup_dir.join(file_name);

    let props = Arc::new(WriterProperties::builder().build());
    let file = std::fs::File::create(&abs_file_name)?;
    let mut writer = SerializedFileWriter::new(file, schema, props)?;

    let mut row_group_writer = writer.next_row_group()?;

    for column in users_describe.columns() {
        let mut column_writer = row_group_writer.next_column()?.unwrap();

        let adds = match column.type_info() {
            type_info if type_info.type_eq(&sqlx::postgres::PgTypeInfo::with_name("INT8")) => {
                let values = users_response
                    .iter()
                    .map(|row| row.get(column.name()))
                    .collect::<Vec<_>>();

                column_writer
                    .typed::<Int64Type>()
                    .write_batch(&values[..], None, None)?
            }
            type_info if type_info.type_eq(&sqlx::postgres::PgTypeInfo::with_name("VARCHAR")) => {
                let values = users_response
                    .iter()
                    .map(|row| {
                        let value: &str = row.get(column.name());
                        // ByteArray::from(value)
                        value.into()
                    })
                    .collect::<Vec<_>>();

                column_writer
                    .typed::<ByteArrayType>()
                    .write_batch(&values[..], None, None)?
            }
            _ => {
                warn!("Unsupported column type: {:?}", column.type_info());
                0
            }
        };

        println!("Added {} values to column {}", adds, column.name());

        column_writer.close()?;
    }

    let row_meta = row_group_writer.close()?;
    debug!("Row group metadata: {:?}", row_meta);

    let meta = writer.close()?;

    println!("Backup file created: {:?}", abs_file_name);
    println!("Metadata: {:?}", meta);

    ct.cancel();

    Ok(())
}

pub fn start_db_backup(config: &PersistenceConfig, backup_dir: &PathBuf) -> Result<(), MyError> {
    let ct = CancellationToken::new();

    let runtime = crate::tokio_tools::ThreadRuntime {
        threads: 0,
        stack_size: 0,
    };

    run_in_tokio("db_backup", &runtime, db_backup(ct, config, backup_dir))
}
