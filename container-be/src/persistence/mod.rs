use std::{path::PathBuf, sync::Arc};

use log::{debug, info, warn};
use parquet::record::RowAccessor;
use parquet::{
    file::{properties::WriterProperties, writer::SerializedFileWriter},
    schema::parser::parse_message_type,
};
use serde::Deserialize;
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, Column, Executor, PgPool, Pool};
use tokio_util::sync::CancellationToken;
use url::Url;

// mod parquet;

use crate::{error::MyError, tokio_tools::run_in_tokio, webserver::users, UrlWithUsernamePassword};
// use sqlx::Row;

#[derive(Deserialize, Debug, Clone)]
pub struct DbConfig {
    pub pool_size: u32,
    pub connection: UrlWithUsernamePassword,
}

impl DbConfig {
    pub fn connection(&self) -> Url {
        self.connection.clone().into()
    }
}

#[derive(Deserialize, Debug, Clone)]
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

    info!("select_reply: {:?}", select_reply);

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

    info!("select_reply: {:?}", select_reply);

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

pub async fn db_backup(
    ct: CancellationToken,
    config: &PersistenceConfig,
    backup_dir: &PathBuf,
) -> Result<(), MyError> {
    let state = PersistenceState::new(config).await?;

    let pool = state.pool_pg.clone();

    let query = "SELECT * FROM users";

    let users_describe = pool.describe(query).await?;
    users_describe.columns().iter().for_each(|col| {
        println!("Column: {:?}", col);
    });

    println!("Describe result: {:?}", users_describe);

    let users_response = sqlx::query("SELECT * FROM users")
        // .bind("users")
        .fetch_all(&pool)
        .await
        .expect("Connect to DB");

    // Run backup
    println!("DB Response: {:?}", users_response);

    let user_schema = "
        message schema {
            REQUIRED INT64 id;
            REQUIRED BINARY forename (UTF8);
            REQUIRED BINARY surname (UTF8);
            REQUIRED BINARY password (UTF8);
        }
    ";

    let schema = Arc::new(parse_message_type(user_schema)?);
    println!("Schema: {:?}", schema);

    let file_name = "backup.parquet";

    let abs_file_name = backup_dir.join(file_name);

    let props = Arc::new(WriterProperties::builder().build());
    let file = std::fs::File::create(&abs_file_name)?;
    let mut writer = SerializedFileWriter::new(file, schema, props)?;

    // Write each user as a row
    let mut row_group_writer = writer.next_row_group()?;
    for column in users_describe.columns() {
        println!("Column: {:?}", column.name());

        match column.name() {
            "id" => {
                println!("Column: {:?}", column.name());
                let mut column = row_group_writer.next_column()?.unwrap();
                let values: &[i64] = &[1, 2, 4, 8];
                column
                    .typed::<parquet::data_type::Int64Type>()
                    .write_batch(&values[..], None, None)?;
                column.close()?;
            }
            "forename" => {
                println!("Column: {:?}", column.name());
                let mut column = row_group_writer.next_column()?.unwrap();
                let values: Vec<parquet::data_type::ByteArray> = vec![
                    parquet::data_type::ByteArray::from("John"),
                    parquet::data_type::ByteArray::from("Jane"),
                    parquet::data_type::ByteArray::from("Doe"),
                    parquet::data_type::ByteArray::from("Smith"),
                ];
                column
                    .typed::<parquet::data_type::ByteArrayType>()
                    .write_batch(&values[..], None, None)?;
                column.close()?;
            }
            "surname" => {
                println!("Column: {:?}", column.name());
                let mut column = row_group_writer.next_column()?.unwrap();
                let values: Vec<parquet::data_type::ByteArray> = vec![
                    parquet::data_type::ByteArray::from("Smith"),
                    parquet::data_type::ByteArray::from("Doe"),
                    parquet::data_type::ByteArray::from("Brown"),
                    parquet::data_type::ByteArray::from("Johnson"),
                ];
                column
                    .typed::<parquet::data_type::ByteArrayType>()
                    .write_batch(&values[..], None, None)?;
                column.close()?;
            }
            "password" => {
                println!("Column: {:?}", column.name());
                let mut column = row_group_writer.next_column()?.unwrap();
                let values: Vec<parquet::data_type::ByteArray> = vec![
                    parquet::data_type::ByteArray::from("password1"),
                    parquet::data_type::ByteArray::from("password2"),
                    parquet::data_type::ByteArray::from("password3"),
                    parquet::data_type::ByteArray::from("password4"),
                ];
                column
                    .typed::<parquet::data_type::ByteArrayType>()
                    .write_batch(&values[..], None, None)?;
                column.close()?;
            }
            _ => {}
        }

        //    row_group_writer
        //        .get_column_writer(column)?
        //        .write_batch(&users_response, None, None)?;
    }
    for user in users_response {
        println!("Processing row {:?}", user);
        //    row.add_field(user.id.map(|id| id.into()).unwrap_or_default());
        //    row.add_field(user.forename.clone().into());
        //    row.add_field(user.surname.clone().into());
        //    row.add_field(user.password.clone().into());
    }
    let row_meta = row_group_writer.close()?;
    println!("Row group metadata: {:?}", row_meta);

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
