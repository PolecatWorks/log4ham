use std::{sync::Arc, ops::Deref};

use log::info;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use url::Url;

use crate::{error::MyError, UrlWithUsernamePassword};


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
    pub async fn new(config: PersistenceConfig) -> Result<PersistenceState, MyError> {
        info!("Creating PersistenceState with config: {:?}",config.db.connection);
        let pool_pg = PgPoolOptions::new()
            .max_connections(config.db.pool_size)
            .connect(config.db.connection().as_str())
            .await?;

        Ok(PersistenceState {
            config,
            pool_pg,
        })
    }
}
