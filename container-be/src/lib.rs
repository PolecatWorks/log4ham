//! A Simple tutorial for Rust
//!
//! The tutorial covers how to setup a simple CLI program.
//! The program will allow the generation or a JSON schema generating data to match the schema.
//! Additionally the program can validate JSON loaded to the application or can run an http server to allow uploading of json file to the application.

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use error::MyError;
use figment::{
    providers::{Format, Yaml},
    Figment,
};
use figment_file_provider_adapter::FileAdapter;
use hams::HamsConfig;
use persistence::{PersistenceConfig, PersistenceState};
use serde::Deserialize;
use tokio_tools::ThreadRuntime;
use url::Url;
use warp::reject::Reject;
use webserver::WebServiceConfig;

pub mod error;
pub mod hams;
pub mod persistence;
pub mod tokio_tools;
pub mod webserver;

/// Name of the Crate
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// Version of the Crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Marker trait to indicate MyError is a planned rejection type
impl Reject for MyError {}

#[derive(Deserialize, Debug, Clone)]
pub struct UrlWithUsernamePassword {
    pub url: Url,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl From<UrlWithUsernamePassword> for Url {
    fn from(value: UrlWithUsernamePassword) -> Self {
        let mut return_url = value.url;

        if let Some(password) = value.password {
            return_url.set_password(Some(&password)).unwrap();
        }
        if let Some(username) = value.username {
            return_url.set_username(&username).unwrap();
        }
        return_url
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct MyConfig {
    /// Config of my web service
    pub hams: HamsConfig,
    pub runtime: ThreadRuntime,
    pub webservice: WebServiceConfig,
    pub persistence: PersistenceConfig,
}

impl MyConfig {
    // Note the `nested` option on both `file` providers. This makes each
    // top-level dictionary act as a profile.
    pub fn figment<P: AsRef<Path> + Clone>(yaml_string: &str, secrets: P) -> Figment {
        Figment::new().merge(FileAdapter::wrap(Yaml::string(yaml_string)).relative_to_dir(secrets))
    }
}

#[derive(Debug, Clone)]
pub struct MyState {
    name: String,
    config: MyConfig,
    db_state: PersistenceState,
    pub count_good: Arc<Mutex<usize>>,
    pub count_fail: Arc<Mutex<usize>>,
}

impl<'a, 'b: 'a> MyState {
    pub async fn new<S: Into<String>>(name: S, config: &MyConfig) -> Result<MyState, MyError> {
        let db_state = PersistenceState::new(&config.persistence).await?;

        Ok(MyState {
            name: name.into(),
            config: config.clone(),
            db_state,
            count_good: Arc::new(Mutex::new(0)),
            count_fail: Arc::new(Mutex::new(0)),
        })
    }
}

#[cfg(test)]
mod test {
    use super::UrlWithUsernamePassword;
    use url::Url;

    #[test]
    fn try_out_enum() {
        let temp_url = UrlWithUsernamePassword {
            url: Url::parse("postgres://myuser:mypass@localhost/mydb").unwrap(),
            username: None,
            password: None,
        };
        assert_eq!(
            Into::<Url>::into(temp_url).as_str(),
            "postgres://myuser:mypass@localhost/mydb"
        );

        let temp_url = UrlWithUsernamePassword {
            url: Url::parse("postgres://myuser:mypass@localhost/mydb").unwrap(),
            username: Some("user0".to_owned()),
            password: Some("pass0".to_owned()),
        };
        assert_eq!(
            Into::<Url>::into(temp_url).as_str(),
            "postgres://user0:pass0@localhost/mydb"
        );
    }
}
