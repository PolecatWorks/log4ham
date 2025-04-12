use parquet_derive::ParquetRecordWriter;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use warp::Filter;

use super::{with_db_pool_pg, DbBigSerial, PageOptions};

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, PartialEq, Clone)]
// #[derive(ParquetRecordWriter)]
pub struct User {
    pub id: Option<DbBigSerial>,
    pub forename: String,
    pub surname: String,
    pub password: String,
}

impl User {
    pub fn new<S: Into<String>>(forename: S, surname: S, password: S) -> Self {
        Self {
            id: None,
            forename: forename.into(),
            surname: surname.into(),
            password: password.into(),
        }
    }
}

impl User {
    /// Compare the content of the User object not the id
    pub fn content_eq(&self, other: &Self) -> bool {
        self.forename == other.forename
            && self.surname == other.surname
            && self.password == other.password
    }
    /// Compare the id of the User object not the content
    pub fn id_eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl warp::Reply for User {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self).into_response()
    }
}

pub fn users_list(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .and(warp::query::<PageOptions>())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::list)
}

pub fn users_create(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        // .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::create)
}

pub fn users_read(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(DbBigSerial)
        .and(warp::get())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::read)
}

pub fn users_update(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(DbBigSerial)
        .and(warp::put())
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::update)
}

pub fn users_delete(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(DbBigSerial)
        .and(warp::delete())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::delete)
}

pub fn users(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    users_list(pool_pg.clone())
        .or(users_create(pool_pg.clone()))
        .or(users_read(pool_pg.clone()))
        .or(users_update(pool_pg.clone()))
        .or(users_delete(pool_pg.clone()))
}

pub mod handlers {
    use super::*;
    use crate::{error::MyError, webserver::ListPages};
    use sqlx::PgPool;

    pub async fn list(options: PageOptions, pool_pg: PgPool) -> Result<ListPages, warp::Rejection> {
        let options = PageOptions::defaulting(options);

        let ids = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(options.size)
        .bind(options.sort)
        .fetch_all(&pool_pg)
        .await
        .map_err(MyError::from)?;

        let list_ids = ListPages {
            pagination: options,
            ids: ids.iter().map(|u| u.id.unwrap()).collect(),
        };

        Ok(list_ids)
    }

    pub async fn create(user: User, pool_pg: PgPool) -> Result<User, warp::Rejection> {
        // let mut conn = pool_pg.acquire().await.map_err(MyError::from)?;

        if user.id.is_some() {
            return Err(warp::reject::custom(MyError::Message("ID must not be set")));
        }

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (forename, surname, password)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&user.forename)
        .bind(&user.surname)
        .bind(&user.password)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

        Ok(user)
    }

    pub async fn read(id: DbBigSerial, pool_pg: PgPool) -> Result<User, warp::Rejection> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

        Ok(user)
    }

    /// Update a user
    ///
    /// This function will update a user in the database
    pub async fn update(
        id: DbBigSerial,
        user: User,
        pool_pg: PgPool,
    ) -> Result<User, warp::Rejection> {
        if user.id.is_none() || id != user.id.unwrap() {
            return Err(MyError::Message("ids on path and body must match for update").into());
        }

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET forename = $2, surname = $3, password = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&user.forename)
        .bind(&user.surname)
        .bind(&user.password)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

        Ok(user)
    }

    /// Delete a user
    ///
    /// This function will delete a user from the database
    pub async fn delete(id: DbBigSerial, pool_pg: PgPool) -> Result<User, warp::Rejection> {
        let user = sqlx::query_as::<_, User>(
            r#"
            DELETE FROM users
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use crate::webserver::ListPages;

    use super::*;

    use sqlx::{PgPool, Row};
    use warp::http::StatusCode;

    // const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

    #[tokio::test]
    async fn test_async_fn() {
        tokio::task::yield_now().await;
    }

    #[sqlx::test()]
    async fn no_users_in_table(pool: PgPool) -> sqlx::Result<()> {
        // let mut conn = pool.acquire().await?;

        let foo = sqlx::query("SELECT count(*) FROM users")
            .fetch_one(&pool)
            .await?;

        assert_eq!(foo.get::<i64, _>(0), 0);

        Ok(())
    }

    // Test that list returns empty list when no users
    #[sqlx::test]
    async fn test_users_list_empty(pool: PgPool) -> sqlx::Result<()> {
        let users_api = users(pool.clone());

        let resp = warp::test::request()
            .method("GET")
            // .path("users")
            .reply(&users_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn test_users_create(pool: PgPool) -> sqlx::Result<()> {
        let users_api = users(pool.clone());

        let user = User::new("John", "Doe", "password");

        let resp = warp::test::request()
            .method("POST")
            // .path("users")
            .json(&user)
            .reply(&users_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let user_resp: User = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(user_resp.forename, user.forename);
        assert_eq!(user_resp.surname, user.surname);
        assert_eq!(user_resp.password, user.password);
        assert!(user_resp.id.is_some());

        // Check that the user is in the database
        let user_db = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE forename = $1
            AND surname = $2
            AND password = $3
            "#,
        )
        .bind(&user.forename)
        .bind(&user.surname)
        .bind(&user.password)
        .fetch_one(&pool)
        .await?;

        assert_eq!(user_db.forename, user.forename);
        assert_eq!(user_db.surname, user.surname);
        assert_eq!(user_db.password, user.password);
        assert!(user_db.id.unwrap() == user_resp.id.unwrap());

        Ok(())
    }

    // Test if we can see the user we created in the list
    #[sqlx::test]
    async fn test_users_list_one_user(pool: PgPool) -> sqlx::Result<()> {
        let users_api = users(pool.clone());

        let user = User::new("John", "Doe", "password");

        let resp = warp::test::request()
            .method("POST")
            .path("/")
            .json(&user)
            .reply(&users_api)
            .await;

        let new_user: User = serde_json::from_slice(resp.body()).unwrap();

        let resp = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&users_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 1);

        assert_eq!(list_ids.ids[0], new_user.id.unwrap());

        Ok(())
    }

    // Test users read with invalid user id
    #[sqlx::test]
    async fn test_users_read_invalid_id(pool: PgPool) -> sqlx::Result<()> {
        let users_api = users(pool.clone());

        let resp = warp::test::request()
            .method("GET")
            .path("/users/IAMNOTAUSERID")
            .reply(&users_api)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    // Test users read with valid user id
    #[sqlx::test]
    async fn test_users_read_valid_id(pool: PgPool) -> sqlx::Result<()> {
        let users_api = users(pool.clone());

        let user = User::new("John", "Doe", "password");

        let resp = warp::test::request()
            .method("POST")
            .path("/")
            .json(&user)
            .reply(&users_api)
            .await;

        let user_post: User = serde_json::from_slice(resp.body()).unwrap();

        let resp = warp::test::request()
            .method("GET")
            .path(format!("/{}", user_post.id.unwrap()).as_str())
            .reply(&users_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let user_resp: User = serde_json::from_slice(resp.body()).unwrap();
        assert!(user_resp.content_eq(&user));
        assert_eq!(user_resp, user_post);

        Ok(())
    }

    // Test update user
    #[sqlx::test]
    async fn test_users_update(pool: PgPool) -> sqlx::Result<()> {
        let users_api = users(pool.clone());

        let user = User::new("John", "Doe", "password");

        let resp = warp::test::request()
            .method("POST")
            .path("/")
            .json(&user)
            .reply(&users_api)
            .await;

        let user_created: User = serde_json::from_slice(resp.body()).unwrap();

        let user_new = User {
            id: user_created.id,
            forename: "Jane".to_string(),
            surname: "Stewart".to_string(),
            password: "password1".to_string(),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path(format!("/{}", user_new.id.unwrap()).as_str())
            .json(&user_new)
            .reply(&users_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let user_updated: User = serde_json::from_slice(resp.body()).unwrap();

        assert_eq!(user_updated, user_new);

        Ok(())
    }

    // Test delete user
    #[sqlx::test]
    async fn test_users_delete(pool: PgPool) -> sqlx::Result<()> {
        let users_api = users(pool.clone());

        let user = User::new("John", "Doe", "password");

        let resp = warp::test::request()
            .method("POST")
            .path("/")
            .json(&user)
            .reply(&users_api)
            .await;

        let user_created: User = serde_json::from_slice(resp.body()).unwrap();

        let resp = warp::test::request()
            .method("DELETE")
            .path(format!("/{}", user_created.id.unwrap()).as_str())
            .reply(&users_api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let user_deleted: User = serde_json::from_slice(resp.body()).unwrap();

        assert_eq!(user_deleted, user_created);

        let resp_list = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&users_api)
            .await;

        assert_eq!(resp_list.status(), StatusCode::OK);
        let list_ids: ListPages = serde_json::from_slice(resp_list.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 0);

        Ok(())
    }
}
