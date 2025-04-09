use log::info;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{error::MyError, tokio_tools::run_in_tokio, UrlWithUsernamePassword};

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
