use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use warp::Filter;

use super::{with_db_pool_pg, DbBigSerial, ListOptions};






#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, PartialEq, Clone)]
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

impl warp::Reply for User {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self).into_response()
    }
}

pub fn users_list(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply, ), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .and(warp::query::<ListOptions>())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::list)
}


pub fn users_create(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply, ), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::create)
}

pub fn users(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply, ), Error = warp::Rejection> + Clone {
    users_list(pool_pg.clone())
        .or(users_create(pool_pg.clone()))
}

pub mod handlers {
    use super::*;
    use crate::{error::MyError, webserver::ListIds};
    use sqlx::PgPool;

    pub async fn list(
        options: ListOptions,
        pool_pg: PgPool,
    ) -> Result<ListIds, warp::Rejection> {
        let ids = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            LIMIT $1 OFFSET $2
            "#)
            .bind(options.limit)
            .bind(options.offset)
            .fetch_all(&pool_pg)
            .await
            .map_err(MyError::from)?;

        let list_ids = ListIds {
            options,
            ids: ids.iter().map(|u| u.id.unwrap()).collect(),
        };

        Ok(list_ids)
    }


    pub async fn create(
        user: User,
        pool_pg: PgPool,
    ) -> Result<User, warp::Rejection> {
        // let mut conn = pool_pg.acquire().await.map_err(MyError::from)?;

        if user.id.is_some() {
            return Err(warp::reject::custom(MyError::Message("ID must not be set")));
        }

        let user = sqlx::query_as::<_, User> (
            r#"
            INSERT INTO users (forename, surname, password)
            VALUES ($1, $2, $3)
            RETURNING *
            "#)
            .bind(&user.forename)
            .bind(&user.surname)
            .bind(&user.password)

            .fetch_one(&pool_pg)
            .await
            .map_err(MyError::from)?;

        Ok(user)
    }
}


#[cfg(test)]
mod tests {
    use crate::webserver::ListIds;

    use super::*;

    use warp::http::StatusCode;
    use sqlx::{PgPool, Row};

    const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

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
        let resp = warp::test::request()
            .method("GET")
            // .path("users")
            .reply(&users(pool))
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let list_ids: ListIds = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 0);

        Ok(())
    }


    #[sqlx::test]
    async fn test_users_create(pool: PgPool) -> sqlx::Result<()> {

        let user = User::new("John", "Doe", "password");

        let resp = warp::test::request()
            .method("POST")
            // .path("users")
            .json(&user)
            .reply(&users(pool.clone()))
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let user_resp: User = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(user_resp.forename, user.forename);
        assert_eq!(user_resp.surname, user.surname);
        assert_eq!(user_resp.password, user.password);

        // Check that the user is in the database
        let user_db = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE forename = $1
            AND surname = $2
            AND password = $3
            "#)
            .bind(&user.forename)
            .bind(&user.surname)
            .bind(&user.password)
            .fetch_one(&pool)
            .await?;

        assert_eq!(user_db.forename, user.forename);
        assert_eq!(user_db.surname, user.surname);
        assert_eq!(user_db.password, user.password);
        assert!(user_db.id.is_some());

        Ok(())
    }

    // Test if we can see the user we created in the list
    #[sqlx::test]
    async fn test_users_list_one_user(pool: PgPool) -> sqlx::Result<()> {

        let user = User::new("John", "Doe", "password");

        let resp = warp::test::request()
            .method("POST")
            // .path("users")
            .json(&user)
            .reply(&users(pool.clone()))
            .await;

        let new_user: User = serde_json::from_slice(resp.body()).unwrap();

        let resp = warp::test::request()
            .method("GET")
            // .path("users")
            .reply(&users(pool.clone()))
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let list_ids: ListIds = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 1);

        assert_eq!(list_ids.ids[0], new_user.id.unwrap());

        Ok(())
    }

}
