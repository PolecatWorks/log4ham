//! A Simple tutorial for Rust
//!
//! The tutorial covers how to setup a simple CLI program.
//! The program will allow the generation or a JSON schema generating data to match the schema.
//! Additionally the program can validate JSON loaded to the application or can run an http server to allow uploading of json file to the application.

use std::{convert::Infallible, sync::Arc};

use error::MyError;
use futures::{Stream, StreamExt};

use app_schema::schema_string;
use log::info;

use serde::{Deserialize, Serialize};

use sqlx::PgPool;
use tokio_tools::run_in_tokio;
use url::Url;
use warp::{hyper::StatusCode, reject::Reject, reply, Rejection, Reply};

use serde_json::{json, Value};

use bytes::Buf;

use crate::app_schema::Person;

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

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, json_message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, reply::json(&"Not Found".to_string()))
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            reply::json(&"Payload too large".to_string()),
        )
    } else if let Some(e) = err.find::<MyError>() {
        match e {
            MyError::Message(detail) => {
                let error_message = json!({
                    "errorType": "Message",
                    "detail": detail,
                });

                (StatusCode::IM_A_TEAPOT, reply::json(&error_message))
            }
            MyError::Cancelled => todo!(),
            MyError::Serde(_) => todo!(),
            MyError::Io(_) => todo!(),
            MyError::JsonValidation(errors) => {
                let myval = serde_json::json!( { "status:": "validation failed","errors": errors});

                (StatusCode::BAD_REQUEST, reply::json(&myval))
            }
            MyError::ValidationError() => todo!(),
            MyError::FigmentError(err) => todo!(),
            MyError::SqlxError(err) => {
                info!("error is {}", err);

                (
                    StatusCode::IM_A_TEAPOT,
                    reply::json(&"DB error".to_string()),
                )
            }
            MyError::PreflightCheck => todo!(),
            MyError::ShutdownCheck => todo!(),
        }
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            reply::json(&"Internal Server Error".to_string()),
        )
    };

    Ok(warp::reply::with_status(json_message, code))
}

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
