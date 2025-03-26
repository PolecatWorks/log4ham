use sqlx::PgPool;
use warp::Filter;

use crate::webserver::{with_db_pool_pg, PageOptions};

use super::{handlers, Contact};

impl warp::Reply for Contact {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self).into_response()
    }
}

pub fn list(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .and(warp::query::<PageOptions>())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::list)
}

pub fn create(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        // .and(warp::path::end())
        .and(warp::post())
        .and(with_db_pool_pg(pool_pg))
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 32))
        .and_then(handlers::create)
}

pub fn read(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::get()
        .and(with_db_pool_pg(pool_pg))
        .and(warp::path::param())
        .and_then(handlers::read)
}

pub fn update(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::put()
        .and(with_db_pool_pg(pool_pg))
        .and(warp::path::param())
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 32))
        .and_then(handlers::update)
}

pub fn delete(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::delete()
        .and(with_db_pool_pg(pool_pg))
        .and(warp::path::param())
        .and_then(handlers::delete)
}

pub fn contacts(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list(pool_pg.clone())
        .or(create(pool_pg.clone()))
        .or(read(pool_pg.clone()))
        .or(update(pool_pg.clone()))
        .or(delete(pool_pg.clone()))
}

#[cfg(test)]
mod tests {
    use sqlx::types::Decimal;

    use crate::webserver::{
        contacts::{Band, Mode},
        users::{self, User},
        DbBigSerial, ListPages,
    };

    use super::*;

    /// Test GET on contacts returns empty list
    #[sqlx::test]
    async fn test_list_empty(pool: PgPool) {
        let api = contacts(pool);

        let resp = warp::test::request()
            .method("GET")
            // .path("/contacts")
            .reply(&api)
            .await;

        assert_eq!(resp.status(), 200);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 0);
    }

    async fn create_user(pool: &PgPool) -> Result<User, warp::Rejection> {
        users::handlers::create(users::User::new("test", "user", "password"), pool.clone()).await
    }

    fn test_contact(user_id: DbBigSerial) -> Contact {
        Contact::new(
            None,
            user_id,
            chrono::NaiveDate::parse_from_str("2023-01-01", "%Y-%m-%d").unwrap(),
            chrono::NaiveTime::parse_from_str("12:00", "%H:%M").unwrap(),
            "CALLSIGN".to_string(),
            "MI7IEU".to_string(),
            Band::B20m,
            Some(Decimal::new(202, 2)),
            Mode::Ssb,
            Some("59".to_string()),
            Some("59".to_string()),
            Some("NAME".to_string()),
            Some("QTH".to_string()),
            Some("GRID".to_string()),
            Some("COUNTRY".to_string()),
            Some("STATE".to_string()),
            Some("COUNTY".to_string()),
            Some("NOTES".to_string()),
            true,
        )
    }

    /// Test POST on contacts creates a new contact
    #[sqlx::test]
    async fn test_create(pool: PgPool) {
        let user = create_user(&pool).await.unwrap();

        let contact_new = test_contact(user.id.unwrap());

        let api = contacts(pool);
        let resp = warp::test::request()
            .method("POST")
            // .path("/contacts")
            .json(&contact_new)
            .reply(&api)
            .await;

        assert_eq!(resp.status(), 200);
        let contact_created: Contact = serde_json::from_slice(resp.body()).unwrap();

        println!("contact_new: {:?}", contact_new);
        println!("contact_created: {:?}", contact_created);

        assert!(contact_created.content_eq(&contact_new));
    }

    /// Test GET on contacts returns error then returns contact after being created
    #[sqlx::test]
    async fn test_read(pool: PgPool) {
        let user = create_user(&pool).await.unwrap();

        let contact_new = test_contact(user.id.unwrap());

        let api = contacts(pool.clone());
        let resp = warp::test::request()
            .method("GET")
            .path(&format!("/{}", 1))
            .reply(&api)
            .await;

        assert_ne!(resp.status(), 200);
        // TODO: check error status should be 404

        let contact_created = handlers::create(pool.clone(), contact_new.clone())
            .await
            .unwrap();

        let contact_read = warp::test::request()
            .method("GET")
            .path(&format!("/{}", contact_created.id.unwrap()))
            .reply(&api)
            .await;

        assert_eq!(contact_read.status(), 200);

        let contact_read: Contact = serde_json::from_slice(contact_read.body()).unwrap();

        assert!(contact_read.content_eq(&contact_new));
    }

    /// Test PUT on contacts updates a contact
    #[sqlx::test]
    async fn test_update(pool: PgPool) {
        let user = create_user(&pool).await.unwrap();

        let contact_new = test_contact(user.id.unwrap());

        let contact_created = handlers::create(pool.clone(), contact_new.clone())
            .await
            .unwrap();

        let mut contact_updated = contact_created.clone();
        contact_updated.callsign = "CALLSIGN2".to_string();

        let api = contacts(pool.clone());
        let contact_updated = warp::test::request()
            .method("PUT")
            .path(&format!("/{}", contact_created.id.unwrap()))
            .json(&contact_updated)
            .reply(&api)
            .await;

        assert_eq!(contact_updated.status(), 200);

        let contact_updated: Contact = serde_json::from_slice(contact_updated.body()).unwrap();

        assert!(contact_updated.content_eq(&contact_updated));
    }

    /// Test DELETE on contacts deletes a contact
    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        let user = create_user(&pool).await.unwrap();

        let contact_new = test_contact(user.id.unwrap());

        let contact_created = handlers::create(pool.clone(), contact_new.clone())
            .await
            .unwrap();

        let api = contacts(pool.clone());
        let contact_deleted = warp::test::request()
            .method("DELETE")
            .path(&format!("/{}", contact_created.id.unwrap()))
            .reply(&api)
            .await;

        assert_eq!(contact_deleted.status(), 200);

        let contact_deleted: Contact = serde_json::from_slice(contact_deleted.body()).unwrap();

        assert!(contact_deleted.content_eq(&contact_created));
    }
}
