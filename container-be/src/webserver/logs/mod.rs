use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use warp::Filter;
mod contacts;
mod handlers;

use super::{with_db_pool_pg, DbBigSerial, PageOptions};

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, PartialEq, Clone)]
pub struct Log {
    pub id: Option<DbBigSerial>,
    pub user_id: DbBigSerial,
    pub description: String,
}

impl Log {
    pub fn new<S: Into<String>>(user_id: DbBigSerial, description: S) -> Self {
        Self {
            id: None,
            user_id,
            description: description.into(),
        }
    }
}

impl Log {
    pub fn content_eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id && self.description == other.description
    }
    pub fn id_eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl warp::Reply for Log {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self).into_response()
    }
}

pub fn logs_list(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .and(warp::query::<PageOptions>())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::list)
}

pub fn logs_create(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::create)
}

pub fn logs_read(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::param::<DbBigSerial>()
        .and(warp::path::end())
        .and(warp::get())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::read)
}

pub fn logs_update(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::param::<DbBigSerial>()
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::update)
}

pub fn logs_delete(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::param::<DbBigSerial>()
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::delete)
}

pub fn logs(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    logs_list(pool_pg.clone())
        .or(logs_create(pool_pg.clone()))
        .or(logs_read(pool_pg.clone()))
        .or(logs_update(pool_pg.clone()))
        .or(logs_delete(pool_pg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webserver::{handle_rejection, users, ListPages};
    use sqlx::PgPool;
    use warp::http::StatusCode;

    #[sqlx::test]
    async fn logs_handler_list_empty(pool: PgPool) -> sqlx::Result<()> {
        let list_ids = handlers::list(PageOptions::default(), pool).await.unwrap();

        assert_eq!(list_ids.ids.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn logs_handler_create(pool: PgPool) -> sqlx::Result<()> {
        let new_user = users::User::new("test", "user", "password");
        let user = users::handlers::create(new_user.clone(), pool.clone())
            .await
            .unwrap();

        let new_log = Log::new(user.id.unwrap(), "test log");

        let log = handlers::create(new_log.clone(), pool.clone())
            .await
            .unwrap();

        assert!(log.content_eq(&new_log));

        let list_ids = handlers::list(PageOptions::default(), pool.clone())
            .await
            .unwrap();
        assert_eq!(list_ids.ids.len(), 1);
        assert_eq!(list_ids.ids[0], log.id.unwrap());

        Ok(())
    }

    #[sqlx::test]
    async fn logs_handler_read(pool: PgPool) -> sqlx::Result<()> {
        let new_user = users::User::new("test", "user", "password");
        let user = users::handlers::create(new_user.clone(), pool.clone())
            .await
            .unwrap();

        // Check that read fails
        let log0_read = handlers::read(0, pool.clone()).await;
        assert!(log0_read.is_err());

        // Create an object
        let log0_new = Log::new(user.id.unwrap(), "test log 0");
        let log0 = handlers::create(log0_new.clone(), pool.clone())
            .await
            .unwrap();

        // Check that read works
        let log0_read = handlers::read(log0.id.unwrap(), pool.clone())
            .await
            .unwrap();
        assert_eq!(log0, log0_read);

        // Create a 2nd object
        let log1_new = Log::new(user.id.unwrap(), "test log 1");
        let log1 = handlers::create(log1_new.clone(), pool.clone())
            .await
            .unwrap();

        // Check that read still works when 2nd object is in place
        let log0_read = handlers::read(log0.id.unwrap(), pool.clone())
            .await
            .unwrap();
        assert_eq!(log0, log0_read);
        let log1_read = handlers::read(log1.id.unwrap(), pool.clone())
            .await
            .unwrap();
        assert_eq!(log1, log1_read);

        Ok(())
    }

    #[sqlx::test]
    async fn logs_list_empty(pool: PgPool) -> sqlx::Result<()> {
        let logs_api = logs(pool);

        let resp = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);

        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();

        assert_eq!(list_ids.ids.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn logs_update(pool: PgPool) -> sqlx::Result<()> {
        let new_user = users::User::new("test", "user", "password");
        let user = users::handlers::create(new_user.clone(), pool.clone())
            .await
            .unwrap();

        let new_log = Log::new(user.id.unwrap(), "test log");
        let log = handlers::create(new_log.clone(), pool.clone())
            .await
            .unwrap();

        let log_update = Log {
            id: log.id,
            user_id: user.id.unwrap(),
            description: "updated log".to_string(),
        };

        let log_updated = handlers::update(log.id.unwrap(), log_update.clone(), pool.clone())
            .await
            .unwrap();

        assert_eq!(log_updated, log_update);

        Ok(())
    }

    #[sqlx::test]
    async fn logs_delete(pool: PgPool) -> sqlx::Result<()> {
        let new_user = users::User::new("test", "user", "password");
        let user = users::handlers::create(new_user.clone(), pool.clone())
            .await
            .unwrap();

        let new_log = Log::new(user.id.unwrap(), "test log");
        let log = handlers::create(new_log.clone(), pool.clone())
            .await
            .unwrap();

        let log_deleted0 = handlers::delete(log.id.unwrap(), pool.clone())
            .await
            .unwrap();

        assert_eq!(log_deleted0, log);

        let list_ids = handlers::list(PageOptions::default(), pool.clone())
            .await
            .unwrap();
        assert_eq!(list_ids.ids.len(), 0);

        let log_deleted1 = handlers::delete(log.id.unwrap(), pool.clone()).await;
        assert!(log_deleted1.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn logs_api_crud(pool: PgPool) -> sqlx::Result<()> {
        let logs_api = logs(pool.clone()).recover(handle_rejection); // Route to handle rejections and get good StatusCode

        let user_new = users::User::new("test", "user", "password");
        let user = users::handlers::create(user_new.clone(), pool.clone())
            .await
            .unwrap();

        let log_new = Log::new(user.id.unwrap(), "test log");

        // List Empty
        let resp = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 0);

        let resp = warp::test::request()
            .method("GET")
            .path(&format!("/{}", 0)) // Guess that we do not have an item of id 0
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // Create
        let resp = warp::test::request()
            .method("POST")
            .path("/")
            .json(&log_new)
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);

        let log: Log = serde_json::from_slice(resp.body()).unwrap();

        assert!(log.content_eq(&log_new));

        // List
        let resp = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 1);

        // Read
        let resp = warp::test::request()
            .method("GET")
            .path(&format!("/{}", log.id.unwrap()))
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let log_read: Log = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(log_read, log);

        // Update
        let log_update = Log {
            id: log.id,
            user_id: log.user_id,
            description: "updated log".to_string(),
        };
        let resp = warp::test::request()
            .method("PUT")
            .path(&format!("/{}", log.id.unwrap()))
            .json(&log_update)
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let log_updated: Log = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(log_updated, log_update);
        assert!(log_updated.id_eq(&log_update));

        let resp = warp::test::request()
            .method("GET")
            .path(&format!("/{}", log.id.unwrap()))
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let log_read: Log = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(log_read, log_update);

        // Delete
        let resp = warp::test::request()
            .method("DELETE")
            .path(&format!("/{}", log.id.unwrap()))
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let log_deleted: Log = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(log_deleted, log_update);

        let resp = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&logs_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 0);

        Ok(())
    }
}
