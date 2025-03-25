pub mod contacts;
pub mod logs;
pub mod users;

use figment::{
    providers::{Format, Yaml},
    Figment,
};
use figment_file_provider_adapter::FileAdapter;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::{
    convert::Infallible,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;
use warp::{
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

use crate::{
    error::MyError,
    hams::{start_hams_api, HamsConfig},
    persistence::{PersistenceConfig, PersistenceState},
    tokio_tools::run_in_tokio,
    NAME,
};

use warp::hyper::StatusCode;

/// Postgres does not support unsigned int so we use i64 to represent the BIGSERIAL type which is a BIGINT in SQL
type DbBigSerial = i64;

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct DbId {
    id: DbBigSerial,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum SortOrder {
    Asc,
    Desc,
}

pub struct PageSort {
    property: String,
    direction: SortOrder,
}

// The query parameters for list_todos.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageOptions {
    pub page: Option<DbBigSerial>,
    pub size: Option<DbBigSerial>,
    #[serde(flatten)]
    pub sort: Option<DbBigSerial>,
}

impl Default for PageOptions {
    fn default() -> Self {
        Self {
            size: Some(5),
            page: Some(0),
            sort: None,
        }
    }
}

impl PageOptions {
    pub fn defaulting(inval: PageOptions) -> PageOptions {
        PageOptions {
            size: if inval.size.is_some() {
                inval.size
            } else {
                PageOptions::default().size
            },
            page: if inval.page.is_some() {
                inval.page
            } else {
                PageOptions::default().page
            },
            sort: if inval.sort.is_some() {
                inval.sort
            } else {
                PageOptions::default().sort
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPages {
    ids: Vec<DbBigSerial>,
    pagination: PageOptions,
}

impl warp::Reply for ListPages {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self).into_response()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebServiceConfig {
    /// Prefix of the served API
    pub prefix: String,
    /// Hostname to start the webservice on
    /// This allows chainging to localhost for dev and 0.0.0.0 or specific address for deployment
    pub address: SocketAddr,
    pub forwarding_headers: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MyConfig {
    /// Config of my web service
    pub hams: HamsConfig,
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
    pub async fn new<S: Into<String>>(name: S, config: MyConfig) -> Result<MyState, MyError> {
        let db_state = PersistenceState::new(config.persistence.clone()).await?;

        Ok(MyState {
            name: name.into(),
            config,
            db_state,
            count_good: Arc::new(Mutex::new(0)),
            count_fail: Arc::new(Mutex::new(0)),
        })
    }
}

pub async fn service_cancellable(ct: CancellationToken, config: MyConfig) -> Result<(), MyError> {
    let state = MyState::new("apple", config).await?;

    let pool_pg = state.db_state.pool_pg.clone();

    // Initialise liveness here

    let hams = tokio::spawn(start_hams_api(state.config.hams.clone(), ct.clone()));

    let client = reqwest::Client::new();

    let server = start_app_api(state.clone(), pool_pg, ct.clone());

    server.await;

    ct.cancel();
    let hams_jh = hams.await.unwrap();
    hams_jh.unwrap();

    Ok(())
}

async fn start_app_api(state: MyState, pool_pg: Pool<Postgres>, ct: CancellationToken) {
    let prefix = state.config.webservice.prefix.clone();

    // Setup http server

    let weblog = warp::log(NAME);

    let combined = warp::path("users")
        .and(users::users(pool_pg.clone()))
        .or(warp::path("logs").and(logs::logs(pool_pg.clone())))
        .recover(handle_rejection)
        .with(weblog);

    let prefix_path = warp::path(prefix.clone());

    let router = prefix_path.and(combined);

    let (addr, server) = warp::serve(router)
        .bind_with_graceful_shutdown(state.config.webservice.address, async move {
            ct.cancelled().await
        });
    info!("Server started on port {}", addr);
    server.await;
}

fn with_db_pool_pg(
    state: Pool<Postgres>,
) -> impl Filter<Extract = (Pool<Postgres>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn with_state(
    state: MyState,
) -> impl Filter<Extract = (MyState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn with_pathbuf(
    pathbuf: PathBuf,
) -> impl Filter<Extract = (PathBuf,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pathbuf.clone())
}

pub fn service_start(config: MyConfig) -> Result<(), MyError> {
    let ct = CancellationToken::new();

    run_in_tokio(service_cancellable(ct, config))
}

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
                // println!("error is {}", err);
                match err {
                    sqlx::Error::RowNotFound => (
                        StatusCode::NOT_FOUND,
                        reply::json(&"Row not found".to_string()),
                    ),
                    _ => (
                        StatusCode::IM_A_TEAPOT,
                        reply::json(&"DB error".to_string()),
                    ),
                }
            }
            MyError::PreflightCheck => todo!(),
            MyError::ShutdownCheck => todo!(),
        }
    } else {
        // eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            reply::json(&"Internal Server Error".to_string()),
        )
    };

    Ok(warp::reply::with_status(json_message, code))
}

#[cfg(test)]
mod tests {
    use sqlx::{PgPool, Row};

    #[sqlx::test(migrations = false)]
    async fn db_connectivity(pool: PgPool) -> sqlx::Result<()> {
        let foo = sqlx::query("SELECT 1").fetch_one(&pool).await?;

        assert_eq!(foo.get::<i32, _>(0), 1);

        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use sqlx::{PgPool, Row};

//     use crate::webserver::{list_files::ListFile, list_versions::ListVersion, lists::List};

//     const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

//     // You could also do `use foo_crate::MIGRATOR` and just refer to it as `MIGRATOR` here.
//     #[sqlx::test]
//     async fn it_gets_a_pool(pool: PgPool) -> sqlx::Result<()> {
//         let mut conn = pool.acquire().await?;

//         let db_name: String = sqlx::query_scalar("SELECT current_database()")
//             .fetch_one(&mut *conn)
//             .await?;

//         assert!(db_name.starts_with("_sqlx_test"), "dbname: {db_name:?}");

//         Ok(())
//     }

//     #[sqlx::test]
//     async fn db_referential_integrity(pool: PgPool) -> sqlx::Result<()> {
//         {
//             // Insert a list
//             let list = sqlx::query_as::<_, List>(
//                 "INSERT INTO lists (name) VALUES ('example0') RETURNING *",
//             )
//             .fetch_one(&pool.clone())
//             .await
//             .expect("Insert List");

//             // Fail Insert of ListVersion if there is not a matching List
//             println!("Check that VersionList cannot be created unless there is a matching List id");
//             let non_matching_id = 7;
//             assert_ne!(non_matching_id, list.id.unwrap());

//             let _version = sqlx::query_as::<_, ListVersion>("INSERT INTO list_versions (version,schema,list) VALUES ('0.0.1', '{}', $1) RETURNING *")
//                 .bind(&non_matching_id)
//                 .fetch_one(&pool.clone()).await
//                 .expect_err("Fail insert of ListVersion");

//             // Add a version to the list
//             let _version = sqlx::query_as::<_, ListVersion>("INSERT INTO list_versions (version,schema,list) VALUES ('0.0.1', '{}', $1) RETURNING *")
//                 .bind(&list.id.unwrap())
//                 .fetch_one(&pool.clone()).await
//                 .expect("Insert Version");

//             println!("Check that List cannot be deleted unless all Versions are deleted");
//             // Fail delete list
//             let _list = sqlx::query_as::<_, List>("DELETE FROM lists WHERE id=$1 RETURNING *")
//                 .bind(&list.id.unwrap())
//                 .fetch_one(&pool.clone())
//                 .await
//                 .expect_err("Fail delete of List");

//             // Delete Version
//             let _version = sqlx::query_as::<_, ListVersion>(
//                 "DELETE FROM list_versions WHERE list=$1 RETURNING *",
//             )
//             .bind(&list.id.unwrap())
//             .fetch_one(&pool.clone())
//             .await
//             .expect("Delete of ListVersion");

//             // Delete List is successful
//             let _list = sqlx::query_as::<_, List>("DELETE FROM lists WHERE id=$1 RETURNING *")
//                 .bind(&list.id.unwrap())
//                 .fetch_one(&pool.clone())
//                 .await
//                 .expect("Delete of List");
//         }

//         {
//             let list0 = sqlx::query_as::<_, List>(
//                 "INSERT INTO lists (name) VALUES ('example0') RETURNING *",
//             )
//             .fetch_one(&pool.clone())
//             .await
//             .expect("Insert List");

//             let version0 = sqlx::query_as::<_, ListVersion>("INSERT INTO list_versions (version,schema,list) VALUES ('0.0.1', '{}', $1) RETURNING *")
//                 .bind(&list0.id.unwrap())
//                 .fetch_one(&pool.clone()).await
//                 .expect("Insert Version");

//             let list1 = sqlx::query_as::<_, List>(
//                 "INSERT INTO lists (name) VALUES ('example1') RETURNING *",
//             )
//             .fetch_one(&pool.clone())
//             .await
//             .expect("Insert List");

//             let version1 = sqlx::query_as::<_, ListVersion>("INSERT INTO list_versions (version,schema,list) VALUES ('0.0.1', '{}', $1) RETURNING *")
//                 .bind(&list1.id.unwrap())
//                 .fetch_one(&pool.clone()).await
//                 .expect("Insert Version");

//             let non_matching_version_id = 7;
//             assert_ne!(non_matching_version_id, version0.id.unwrap());
//             assert_ne!(non_matching_version_id, version1.id.unwrap());

//             println!("Check that active cannot be set unless id of ListVersion is valid");
//             let _version = sqlx::query_as::<_, List>(
//                 "UPDATE lists SET (active) = ($2) WHERE id= $1 RETURNING *",
//             )
//             .bind(&list0.id.unwrap())
//             .bind(&non_matching_version_id)
//             .fetch_one(&pool.clone())
//             .await
//             .expect_err("Fail List update of active because id of ListVersion is not valid");

//             println!(
//                 "Check that active cannot be set if list of ListVersion does not match id of List"
//             );
//             let _version = sqlx::query_as::<_, List>("UPDATE lists SET (active) = ($2) WHERE id= $1 RETURNING *")
//                 .bind(&list0.id.unwrap())
//                 .bind(&version1.id.unwrap())
//                 .fetch_one(&pool.clone()).await
//                 .expect_err("Fail List update of active bcause id of ListVersion does not have a matchi list entry");

//             println!("Update active of list0 to match version0");
//             let _list =
//                 sqlx::query_as::<_, List>("UPDATE lists SET active = $2 WHERE id= $1 RETURNING *")
//                     .bind(&list0.id.unwrap())
//                     .bind(&version0.id.unwrap())
//                     .fetch_one(&pool.clone())
//                     .await
//                     .expect("Update active when matching ListVersion");

//             println!("Cannot delete ListVersion that is references in active");
//             let _version = sqlx::query_as::<_, ListVersion>(
//                 "DELETE FROM list_versions WHERE list=$1 RETURNING *",
//             )
//             .bind(&list0.id.unwrap())
//             .fetch_one(&pool.clone())
//             .await
//             .expect_err("Cannot delete ListVersion used in active on List");

//             println!("Disable active for a list");
//             let _list = sqlx::query_as::<_, List>(
//                 "UPDATE lists SET active = null WHERE id= $1 RETURNING *",
//             )
//             .bind(&list0.id.unwrap())
//             .fetch_one(&pool.clone())
//             .await
//             .expect("Unset active for List");

//             let _version =
//                 sqlx::query_as::<_, ListVersion>("DELETE FROM list_versions RETURNING *")
//                     .fetch_one(&pool.clone())
//                     .await
//                     .expect("Delete of ListVersion");
//             let _list = sqlx::query_as::<_, List>("DELETE FROM lists RETURNING *")
//                 .fetch_one(&pool.clone())
//                 .await
//                 .expect("Delete of List");
//         }

//         {
//             let non_matching_id = 7;

//             println!("Cannot create ListFile that does not have a version");
//             let _version = sqlx::query_as::<_, ListFile>(
//                 "INSERT INTO list_files (version, validated) VALUES ($1, false) RETURNING *",
//             )
//             .bind(&non_matching_id)
//             .fetch_one(&pool.clone())
//             .await
//             .expect_err("Cannot create ListFile if no valid version to link");

//             let list0 = sqlx::query_as::<_, List>(
//                 "INSERT INTO lists (name) VALUES ('example0') RETURNING *",
//             )
//             .fetch_one(&pool.clone())
//             .await
//             .expect("Insert List");

//             let version0 = sqlx::query_as::<_, ListVersion>("INSERT INTO list_versions (version,schema,list) VALUES ('0.0.1', '{}', $1) RETURNING *")
//                 .bind(&list0.id.unwrap())
//                 .fetch_one(&pool.clone()).await
//                 .expect("Insert Version");

//             let file0 = sqlx::query_as::<_, ListFile>(
//                 "INSERT INTO list_files (version, validated) VALUES ($1, false) RETURNING *",
//             )
//             .bind(&version0.id.unwrap())
//             .fetch_one(&pool.clone())
//             .await
//             .expect("Insert File when version is available");

//             let _version =
//                 sqlx::query_as::<_, ListVersion>("DELETE FROM list_versions RETURNING *")
//                     .bind(&version0.id.unwrap())
//                     .fetch_one(&pool.clone())
//                     .await
//                     .expect_err("Fail to delete ListVersion as is referenced by File");

//             let _file0 =
//                 sqlx::query_as::<_, ListFile>("DELETE FROM list_files WHERE id=$1 RETURNING *")
//                     .bind(&file0.id.unwrap())
//                     .fetch_one(&pool.clone())
//                     .await
//                     .expect("Delete file0");

//             let _version0 = sqlx::query_as::<_, ListVersion>(
//                 "DELETE FROM list_versions WHERE id=$1 RETURNING *",
//             )
//             .bind(&version0.id.unwrap())
//             .fetch_one(&pool.clone())
//             .await
//             .expect("Delete version0");

//             let _list0 = sqlx::query_as::<_, List>("DELETE FROM lists WHERE id=$1 RETURNING *")
//                 .bind(&list0.id.unwrap())
//                 .fetch_one(&pool.clone())
//                 .await
//                 .expect("Delete list0");
//         }

//         Ok(())
//     }
// }
