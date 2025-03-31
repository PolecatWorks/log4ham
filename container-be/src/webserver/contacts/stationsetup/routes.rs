use sqlx::PgPool;
use warp::Filter;

use crate::{
    error::MyError,
    webserver::{with_db_pool_pg, DbBigSerial, DbId, ListPages, PageOptions},
};

use super::{handlers, StationSetup};

impl warp::Reply for StationSetup {
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
        .and_then(|options, pool_pg| async move {
            let options = PageOptions::defaulting(options);

            let ids = sqlx::query_as::<_, DbId>(
                "SELECT id FROM station_setup WHERE id > $1 ORDER BY id LIMIT $2",
            )
            .bind(options.page.unwrap())
            .bind(options.size.unwrap())
            .fetch_all(&pool_pg)
            .await
            .map_err(MyError::from)?;

            let list_ids = ListPages {
                pagination: options,
                ids: ids.iter().map(|u| u.id).collect(),
            };

            Ok::<_, warp::Rejection>(list_ids)
        })
}

/// Create a new station setup
pub fn create(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db_pool_pg(pool_pg))
        .and_then(|station_setup: StationSetup, pool_pg| async move {
            let record = sqlx::query_as_with (
                r#"
                INSERT INTO station_setup (contact_id, radio_model, antenna_type, power_output, other_equipment)
                VALUES ($2, $3, $4, $5, $6)
                RETURNING *
                "#,
                station_setup,
            )
            .fetch_one(&pool_pg)
            .await
            .map_err(MyError::from)?;

            Ok::<StationSetup, warp::Rejection>(record)
        })
}

/// Read a station setup
pub fn read(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::get()
        .and(with_db_pool_pg(pool_pg))
        .and(warp::path::param())
        .and_then(|pool_pg, id: i64| async move {
            let record =
                sqlx::query_as::<_, StationSetup>("SELECT * FROM station_setup WHERE id = $1")
                    .bind(id)
                    .fetch_one(&pool_pg)
                    .await
                    .map_err(MyError::from)?;

            Ok::<StationSetup, warp::Rejection>(record)
        })
}

/// Update a station setup
pub fn update(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::put()
        .and(with_db_pool_pg(pool_pg))
        .and(warp::path::param())
        .and(warp::body::json())
        .and_then(|pool_pg, id: DbBigSerial, record: StationSetup| async move {
            if record.id.is_none() || id != record.id.unwrap() {
                return Err(MyError::Message("ids on path and body must match for update").into());
            }

            let record = sqlx::query_as_with(
                "UPDATE station_setup SET contact_id = $2, radio_model = $3, antenna_type = $4, power_output = $5, other_equipment = $6 WHERE id = $1 RETURNING *",
                record,
            )
                // .bind(id)
                // .bind(station_setup.contact_id)
                // .bind(station_setup.radio_model)
                // .bind(station_setup.antenna_type)
                // .bind(station_setup.power_output)
                // .bind(station_setup.other_equipment)
                .fetch_one(&pool_pg)
                .await
                .map_err(MyError::from)?;

            Ok::<StationSetup, warp::Rejection>(record)
        })
}

/// Delete a station setup
pub fn delete(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::delete()
        .and(with_db_pool_pg(pool_pg))
        .and(warp::path::param())
        .and_then(|pool_pg, id: DbBigSerial| async move {
            let record = sqlx::query_as::<_, StationSetup>(
                "DELETE FROM station_setup WHERE id = $1 RETURNING *",
            )
            .bind(id)
            .fetch_one(&pool_pg)
            .await
            .map_err(MyError::from)?;

            Ok::<StationSetup, warp::Rejection>(record)
        })
}

pub fn station_setup(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list(pool_pg.clone())
        .or(create(pool_pg.clone()))
        .or(read(pool_pg.clone()))
        .or(update(pool_pg.clone()))
        .or(delete(pool_pg))
}

#[cfg(test)]
mod tests {
    use sqlx::types::Decimal;

    use crate::webserver::{contacts::stationsetup::StationSetupBuilder, handle_rejection};

    use super::*;
    use crate::webserver::test::create_contact;

    /// Test the list function returns empty list when no records exist
    #[sqlx::test]
    async fn test_list_empty(pool: PgPool) {
        let api = station_setup(pool.clone());

        let response = warp::test::request().method("GET").reply(&api).await;

        assert_eq!(response.status(), 200);
        let body: ListPages = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.ids.len(), 0);
        assert_eq!(body.pagination.size, PageOptions::default().size);
        assert_eq!(body.pagination.page, PageOptions::default().page);
    }

    /// Test the list function setting the page size
    #[sqlx::test]
    async fn test_list_page_size(pool: PgPool) {
        let api = station_setup(pool.clone());

        let response = warp::test::request()
            .method("GET")
            .path("/?page=6")
            // .path("/?page=5&size=6")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200);
        let body: ListPages = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.pagination.page, Some(6));
        assert_eq!(body.pagination.size, PageOptions::default().size);

        let response = warp::test::request()
            .method("GET")
            .path("/?size=7")
            // .path("/?page=5&size=6")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200);
        let body: ListPages = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.pagination.size, Some(7));
        assert_eq!(body.pagination.page, PageOptions::default().page);

        let response = warp::test::request()
            .method("GET")
            .path("/?page=6&size=7")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200);
        let body: ListPages = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.pagination.page, Some(6));
        assert_eq!(body.pagination.size, Some(7));
    }

    /// Test the create, read update and delete functions
    #[sqlx::test]
    async fn test_create(pool: PgPool) {
        let api = station_setup(pool.clone());

        let contact = create_contact(pool.clone()).await;

        let station_setup = StationSetupBuilder::default()
            .contact_id(1)
            .build()
            .unwrap();

        let response = warp::test::request()
            .method("POST")
            .json(&station_setup)
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200, "Response: {:?}", response);
        let created_station_setup: StationSetup = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(created_station_setup.contact_id, 1);

        /// Read the record
        let response = warp::test::request()
            .method("GET")
            .path(&format!("/{}", created_station_setup.id.unwrap()))
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200, "Response: {:?}", response);
        let read_station_setup: StationSetup = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(read_station_setup.id, created_station_setup.id);

        /// Update the record
        let mut updated_station_setup = created_station_setup.clone();
        updated_station_setup.radio_model = Some("Updated Radio Model".to_string());
        updated_station_setup.antenna_type = Some("Updated Antenna Type".to_string());
        updated_station_setup.power_output = Some(Decimal::new(200, 0));
        updated_station_setup.other_equipment = Some("Updated Other Equipment".to_string());
        let response = warp::test::request()
            .method("PUT")
            .path(&format!("/{}", created_station_setup.id.unwrap()))
            .json(&updated_station_setup)
            .reply(&api)
            .await;
        assert_eq!(response.status(), 200, "Response: {:?}", response);
        let updated_station_setup_response: StationSetup =
            serde_json::from_slice(response.body()).unwrap();
        assert_eq!(updated_station_setup_response, updated_station_setup);

        /// Delete the record
        let response = warp::test::request()
            .method("DELETE")
            .path(&format!("/{}", created_station_setup.id.unwrap()))
            .reply(&api)
            .await;
        assert_eq!(response.status(), 200, "Response: {:?}", response);
        let response = warp::test::request()
            .method("GET")
            .path(&format!("/{}", created_station_setup.id.unwrap()))
            .reply(&api.recover(handle_rejection))
            .await;
        assert_eq!(response.status(), 404, "Response: {:?}", response);
    }

    /// Add 10 records to the DB then list with a page size of 2 and iterate over the pages
    #[sqlx::test]
    async fn test_list_with_records(pool: PgPool) {
        let api = station_setup(pool.clone());

        let contact = create_contact(pool.clone()).await;

        for i in 1..=10 {
            let station_setup = StationSetupBuilder::default()
                .contact_id(contact.id.unwrap())
                .radio_model(Some(format!("Radio Model {}", i)))
                .antenna_type(Some(format!("Antenna Type {}", i)))
                .power_output(Some(Decimal::new(i * 10, 0)))
                .other_equipment(Some(format!("Other Equipment {}", i)))
                .build()
                .unwrap();

            let response = warp::test::request()
                .method("POST")
                .json(&station_setup)
                .reply(&api)
                .await;

            assert_eq!(response.status(), 200, "Response: {:?}", response);
        }

        let response = warp::test::request()
            .method("GET")
            .path("/?size=2")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200, "Response: {:?}", response);
        let body: ListPages = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.ids.len(), 2);
        assert_eq!(body.pagination.size, Some(2));
        assert_eq!(body.pagination.page, Some(0));

        // Iterate over the pages
        for page in 2..=5 {
            let response = warp::test::request()
                .method("GET")
                .path(&format!("/?page={}&size=2", page))
                .reply(&api)
                .await;

            assert_eq!(response.status(), 200, "Response: {:?}", response);
            let body: ListPages = serde_json::from_slice(response.body()).unwrap();
            assert_eq!(body.ids.len(), 2);
            assert_eq!(body.pagination.size, Some(2));
            assert_eq!(body.pagination.page, Some(page));
        }
    }
}
