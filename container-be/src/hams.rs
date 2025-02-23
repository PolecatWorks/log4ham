use log::{error, info};
use reqwest::Client;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DurationSeconds;
use std::{net::SocketAddr, time::Duration};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use url::Url;
use warp::Filter;

use crate::error::MyError;

#[derive(Deserialize, Debug, Clone)]
pub struct HamsConfig {
    /// Hostname to start the webservice on
    /// This allows chainging to localhost for dev and 0.0.0.0 or specific address for deployment
    pub address: SocketAddr,
    /// Prefix of the served API
    pub prefix: String,
}

pub async fn start_hams_api(config: HamsConfig, ct: CancellationToken) -> Result<(), MyError> {
    let weblog = warp::log("hams");

    let router = warp::any()
        .map(|| "Hello from Hams".to_string())
        .with(weblog);

    let (addr, server) = warp::serve(router).bind_with_graceful_shutdown(
        config.address,
        // ([0, 0, 0, 0], 8080),
        async move { ct.cancelled().await },
    );
    info!("Hams started on port {}", addr);

    server.await;
    Ok(())
}

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct Checks {
    #[serde_as(as = "DurationSeconds<u64>")]
    pub timeout: Duration,
    pub fails: u32,
    pub preflights: Vec<Url>,
    pub shutdowns: Vec<Url>,
}

impl Checks {
    pub async fn preflight(&self, client: &Client) -> Result<u32, MyError> {
        let mut fails = self.fails;
        for preflight in self.preflights.iter() {
            info!("Checking preflight: {}", preflight);
            while fails > 0 && client.get(preflight.clone()).send().await.is_err() {
                info!(
                    "Failed preflight: {} retrying in {} secs (fail count {}/{})",
                    preflight,
                    self.timeout.as_secs(),
                    fails,
                    self.fails
                );
                sleep(self.timeout).await;
                fails -= 1;
            }
        }
        if fails > 0 {
            info!("Preflight success, {} retries remaining", fails);
            Ok(fails)
        } else {
            error!("Preflight FAIL");
            Err(MyError::PreflightCheck)
        }
    }
    pub async fn shutdown(&self, client: &Client) -> Result<u32, MyError> {
        let mut fails = self.fails;
        for shutdown in self.shutdowns.iter() {
            info!("Checking shutdown: {}", shutdown);
            while fails > 0 && client.get(shutdown.clone()).send().await.is_err() {
                info!(
                    "Failed shutdown: {} retrying in {} secs (fail count {}/{})",
                    shutdown,
                    self.timeout.as_secs(),
                    fails,
                    self.fails
                );
                sleep(self.timeout).await;
                fails -= 1;
            }
        }
        if fails > 0 {
            info!("Shutdown success, {} retries remaining", fails);
            Ok(fails)
        } else {
            error!("Shutdown FAIL");
            Err(MyError::ShutdownCheck)
        }
    }
}
