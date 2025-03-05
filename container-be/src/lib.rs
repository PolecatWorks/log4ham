//! A Simple tutorial for Rust
//!
//! The tutorial covers how to setup a simple CLI program.
//! The program will allow the generation or a JSON schema generating data to match the schema.
//! Additionally the program can validate JSON loaded to the application or can run an http server to allow uploading of json file to the application.

use bytes::Buf;
use error::MyError;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use url::Url;
use warp::{reject::Reject, Rejection, Reply};

pub mod app_schema;
pub mod error;
pub mod hams;
pub mod persistence;
mod tokio_tools;
pub mod webserver;

/// Name of the Crate
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// Version of the Crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

async fn receive_binary(
    mut body: impl Stream<Item = Result<impl Buf, warp::Error>> + Unpin + Send + Sync,
) -> Result<impl Reply, Rejection> {
    // https://github.com/seanmonstar/warp/issues/448
    // eprintln!("Got a file upload of ");
    let mut chunk_tot = 0;
    while let Some(buf) = body.next().await {
        let mut buf = buf.unwrap();
        while buf.remaining() > 0 {
            let chunk = buf.chunk();
            let chunk_len = chunk.len();
            // println!("getting chunk of len = {chunk_len}");
            chunk_tot += chunk_len;
            buf.advance(chunk_len);
        }
    }
    Ok(format!("Upload size = {chunk_tot}"))
}
#[derive(Serialize)]
struct ValidationReply {
    size: usize,
    length: usize,
    validate: bool,
}

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
