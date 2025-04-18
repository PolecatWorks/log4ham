//! A Simple tutorial for Rust
//!
//! The tutorial covers how to setup a simple CLI program.
//! The program will allow the generation or a JSON schema generating data to match the schema.
//! Additionally the program can validate JSON loaded to the application or can run an http server to allow uploading of json file to the application.

use std::{
    ffi::c_void,
    sync::{Arc, Mutex},
};

use config::MyConfig;
use error::MyError;
use hams::start_hams_api;
use hamsrs::Hams;
use metrics::{prometheus_response, prometheus_response_free};
use persistence::PersistenceState;
use prometheus::{IntGauge, Registry};

use tokio_util::sync::CancellationToken;
use warp::reject::Reject;
use webserver::start_app_api;

pub mod config;
pub mod error;
pub mod hams;
mod metrics;
pub mod persistence;
pub mod tokio_tools;
pub mod webserver;

/// Name of the Crate
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// Version of the Crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Marker trait to indicate MyError is a planned rejection type
impl Reject for MyError {}

#[derive(Debug, Clone)]
pub struct MyState {
    name: String,
    config: MyConfig,
    db_state: PersistenceState,
    pub count_good: Arc<Mutex<usize>>,
    pub count_fail: Arc<Mutex<usize>>,
    registry: Registry,
    hello_counter: IntGauge,
}

impl<'a, 'b: 'a> MyState {
    pub async fn new<S: Into<String>>(name: S, config: &MyConfig) -> Result<MyState, MyError> {
        let db_state = PersistenceState::new(&config.persistence).await?;

        let registry = Registry::new();

        let hello_counter = IntGauge::new("my_counter", "A counter for my application")?;

        registry.register(Box::new(hello_counter.clone()))?;

        Ok(MyState {
            name: name.into(),
            config: config.clone(),
            db_state,
            count_good: Arc::new(Mutex::new(0)),
            count_fail: Arc::new(Mutex::new(0)),
            registry,
            hello_counter,
        })
    }
}

pub async fn service_cancellable(ct: CancellationToken, config: &MyConfig) -> Result<(), MyError> {
    let state = MyState::new("apple", config).await?;

    let pool_pg = state.db_state.pool_pg.clone();

    // Initialise liveness here

    let mut config = state.config.hams.clone();

    config.name = NAME.to_owned();
    config.version = VERSION.to_owned();

    let hams2 = Hams::new(ct.clone(), &config).unwrap();

    hams2.register_prometheus(
        prometheus_response,
        prometheus_response_free,
        &state.registry as *const _ as *const c_void,
    )?;

    hams2.start().unwrap();

    let hams = tokio::spawn(start_hams_api(state.config.hams.clone(), ct.clone()));

    let server = start_app_api(state.clone(), pool_pg, ct.clone());

    server.await;

    hams2.stop().unwrap();
    hams2.deregister_prometheus()?;

    ct.cancel();
    let hams_jh = hams.await.unwrap();
    hams_jh.unwrap();

    Ok(())
}
