use sqlx::PgPool;
use warp::Filter;

use crate::webserver::{with_db_pool_pg, PageOptions};

use super::{handlers, QslCard};

impl warp::Reply for QslCard {
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

/// Create a new QslCard
pub fn create(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::post())
        .and(with_db_pool_pg(pool_pg))
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 32))
        .and_then(handlers::create)
}

/// Read a QslCard
pub fn read(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::get()
        .and(with_db_pool_pg(pool_pg))
        .and(warp::path::param())
        .and_then(handlers::read)
}

/// Update a QslCard
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

/// Delete a QslCard
pub fn delete(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::delete()
        .and(with_db_pool_pg(pool_pg))
        .and(warp::path::param())
        .and_then(handlers::delete)
}

/// REST definiton of QslCard
pub fn qsl(
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

    use chrono::NaiveDate;
    use sqlx::types::Decimal;

    use crate::webserver::{
        contacts::{self, Band, Contact, Mode}, handle_rejection, qslcard::QslCardBuilder, users, ListPages
    };

    use super::*;

    async fn create_contact(pool: PgPool) -> Contact {
        let user =
            users::handlers::create(users::User::new("test", "user", "password"), pool.clone())
                .await
                .unwrap();

        println!("user = {:?}", user);

        contacts::handlers::create(
            pool.clone(),
            Contact::new(
                None,
                user.id.unwrap(),
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
            ),
        )
        .await
        .unwrap()
    }

    /// Test GET on contacts returns empty list
    #[sqlx::test]
    async fn test_list_empty(pool: PgPool) {
        let api = qsl(pool);

        let resp = warp::test::request().method("GET").reply(&api).await;

        assert_eq!(resp.status(), 200);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 0);
    }

    /// Create a new QslCard then check it is part of the list then delete it and confirm it is erased
    #[sqlx::test]
    async fn test_create_list_delete(pool: PgPool) {
        let contact = create_contact(pool.clone()).await;

        let api = qsl(pool.clone());

        let qsl_card = QslCardBuilder::default()
            .contact_id(contact.id.unwrap())
            .qsl_sent_date(Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()))
            .build()
            .unwrap();

        let resp = warp::test::request()
            .method("POST")
            .json(&qsl_card)
            .reply(&api)
            .await;

        println!("resp = {:?}", resp);
        assert_eq!(resp.status(), 200);
        let created_qsl_card: QslCard = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(
            created_qsl_card.qsl_sent_date.unwrap(),
            NaiveDate::parse_from_str("2023-01-01", "%Y-%m-%d").unwrap()
        );

        let resp = warp::test::request().method("GET").reply(&api).await;

        assert_eq!(resp.status(), 200);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 1);

        let resp_read = warp::test::request()
            .method("GET")
            .path(&format!("/{}", created_qsl_card.id.unwrap()))
            .reply(&api)
            .await;
        assert_eq!(resp_read.status(), 200);
        let read_qsl_card: QslCard = serde_json::from_slice(resp_read.body()).unwrap();
        assert_eq!(created_qsl_card, read_qsl_card);

        let mut updated_qsl_card = created_qsl_card.clone();
        updated_qsl_card.qsl_sent_date = Some(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap());

        let resp_update = warp::test::request()
            .method("PUT")
            .path(&format!("/{}", created_qsl_card.id.unwrap()))
            .json(&updated_qsl_card)
            .reply(&api)
            .await;

        assert_eq!(resp_update.status(), 200);
        let resp_updated_qsl_card: QslCard = serde_json::from_slice(resp_update.body()).unwrap();
        assert_eq!(updated_qsl_card, resp_updated_qsl_card);

        let resp_delete = warp::test::request()
            .method("DELETE")
            .path(&format!("/{}", created_qsl_card.id.unwrap()))
            .reply(&api)
            .await;

        assert_eq!(resp.status(), 200);

        let resp = warp::test::request().method("GET").reply(&api).await;
        assert_eq!(resp.status(), 200);
        let list_ids: ListPages = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(list_ids.ids.len(), 0);
    }

    /// read a Qsl Card that does not exist and get a 404
    #[sqlx::test]
    async fn test_read_not_found(pool: PgPool) {
        let api = qsl(pool.clone()).recover(handle_rejection);

        let resp = warp::test::request()
            .method("GET")
            .path("/9999")
            .reply(&api)
            .await;

        assert_eq!(
            resp.status(),
            404,
            "Expected 404 for non-existent QslCard, {:?}",
            resp.body()
        );
    }

    /// Delete a QslCard that does not exist and get a 404
    #[sqlx::test]
    async fn test_delete_not_found(pool: PgPool) {
        let api = qsl(pool.clone()).recover(handle_rejection);

        let resp = warp::test::request()
            .method("DELETE")
            .path("/9999")
            .reply(&api)
            .await;

        assert_eq!(
            resp.status(),
            404,
            "Expected 404 for non-existent QslCard, {:?}",
            resp.body()
        );
    }

    /// Update a QslCard that does not exist and get a 404
    #[sqlx::test]
    async fn test_update_not_found(pool: PgPool) {
        let api = qsl(pool.clone()).recover(handle_rejection);

        let qsl_card = QslCardBuilder::default()
            .id(Some(9999))
            .contact_id(1)
            .build()
            .unwrap();

        let resp = warp::test::request()
            .method("PUT")
            .path("/9999")
            .json(&qsl_card)
            .reply(&api)
            .await;

        assert_eq!(
            resp.status(),
            404,
            "Expected 404 for non-existent QslCard, {:?}",
            resp.body()
        );
    }
}
