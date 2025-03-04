use super::{DbBigSerial, ListOptions};
use crate::webserver::with_db_pool_pg;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use warp::{reject::Rejection, Filter};

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, PartialEq, Clone)]
pub struct List {
    pub id: Option<DbBigSerial>,
    pub name: String,
    pub active: Option<DbBigSerial>,
}

impl List {
    pub fn new<S: Into<String>>(name: S) -> List {
        List {
            id: None,
            name: name.into(),
            active: None,
        }
    }
}

impl warp::Reply for List {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self).into_response()
    }
}

#[derive(Serialize)]
struct ListReply {
    id: DbBigSerial,
    message: String,
}

/// POST /lists with JSON body
pub fn lists_create(
    pool_pg: PgPool,
    upload_limit: u64,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::post())
        .and(warp::body::content_length_limit(upload_limit))
        .and(warp::body::json::<List>())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::create)
}

/// GET /lists?offset=3&limit=5
pub fn lists_list(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .and(warp::query::<ListOptions>())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::list)
}

/// GET /lists/:id
pub fn lists_read(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(DbBigSerial)
        .and(warp::get())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::read)
}

/// PUT /lists/:id with JSON body
pub fn lists_update(
    pool_pg: PgPool,
    upload_limit: u64,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(DbBigSerial)
        .and(warp::put())
        .and(warp::body::content_length_limit(upload_limit))
        .and(warp::body::json::<List>())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::update)
}

/// DELETE /lists/:id
pub fn lists_delete(
    pool_pg: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(DbBigSerial)
        .and(warp::delete())
        .and(with_db_pool_pg(pool_pg))
        .and_then(handlers::delete)
}

pub fn lists(
    pool_pg: PgPool,
    upload_limit: u64,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // lists_create(pool_pg.clone())
    lists_list(pool_pg.clone())
        // .or(lists_create(pool_pg.clone(), upload_limit))
        .or(lists_create(pool_pg.clone(), upload_limit))
        .or(lists_read(pool_pg.clone()))
        .or(lists_update(pool_pg.clone(), upload_limit))
        .or(lists_delete(pool_pg))
}

/// handlers for the list APIs
pub mod handlers {
    use super::*;
    use crate::{
        error::MyError,
        webserver::{DbId, ListIds},
    };
    use log::info;
    use sqlx::PgPool;

    /// Create a new list
    ///
    /// Checks that the provided body meets the schema for [List] then creates the records in the lists table returning the id
    /// If the list name already exists then returns an error.
    /// If the
    pub async fn create(body: List, pool_pg: PgPool) -> Result<List, Rejection> {
        // let mut my_list: List = serde_json::from_value(body).map_err(|err| MyError::from(err))?;

        // If id is provided then reply with an error
        if body.id.is_some() {
            return Err(MyError::Message("id must not be provided when creating list").into());
        }

        let list = sqlx::query_as::<_, List>("INSERT INTO lists (name) VALUES ($1) RETURNING *")
            .bind(&body.name)
            .fetch_one(&pool_pg)
            .await
            .map_err(MyError::from)?;

        info!("Create list id={}", list.id.unwrap());

        Ok(list)
    }

    pub async fn list(options: ListOptions, pool_pg: PgPool) -> Result<ListIds, Rejection> {
        let options = ListOptions::defaulting(options);

        // TODO: Add ListOptions to the SQL function
        let ids =
            sqlx::query_as::<_, DbId>("SELECT id FROM lists WHERE id >= $1 ORDER BY id LIMIT $2")
                .bind(options.offset.unwrap())
                .bind(options.limit.unwrap())
                .fetch_all(&pool_pg)
                .await
                .map_err(|e| MyError::from(e))?;

        let ids: Vec<_> = ids.iter().map(|val| val.id).collect();

        let list_ids = ListIds { options, ids };

        info!("Read lists");

        Ok(list_ids)
    }

    pub async fn read(id: DbBigSerial, pool_pg: PgPool) -> Result<List, Rejection> {
        let row = sqlx::query_as::<_, List>("SELECT * FROM lists WHERE id=$1")
            .bind(&id)
            .fetch_one(&pool_pg)
            .await
            .map_err(|e| MyError::from(e))?;

        info!("Read list id={}", id);

        Ok(row)
    }

    /// Update record on lists table
    ///
    /// As the record is pushed the path id is confirmed to match to the updated record object. Both must match or an error ir raised.
    /// The content of the record is updated by this action. Only name and active are updated. It is not valid to change the id.
    pub async fn update(id: DbBigSerial, body: List, pool_pg: PgPool) -> Result<List, Rejection> {
        if body.id.is_none() || id != body.id.unwrap() {
            return Err(MyError::Message("ids on path and body must match for update").into());
        }

        let update = sqlx::query_as::<_, List>(
            "UPDATE lists SET (name, active ) = ($1, $2) WHERE id= $3 RETURNING *",
        )
        .bind(&body.name)
        .bind(&body.active)
        .bind(&body.id)
        .fetch_one(&pool_pg)
        .await
        .map_err(|e| MyError::from(e))?;

        info!("Update list id={}", id);

        Ok(update)
    }

    /// Delete record from lists table
    ///
    /// Posgres ensures referential integrity with respect to list_versions by ensuring that we do cannot delete a
    /// list if it still contains versions.
    /// This forces us to delete the versions belonging to the list before the list itself can be deleted.
    pub async fn delete(id: DbBigSerial, pool_pg: PgPool) -> Result<List, Rejection> {
        let row = sqlx::query_as::<_, List>("DELETE FROM lists WHERE id=$1 RETURNING *")
            .bind(&id)
            .fetch_one(&pool_pg)
            .await
            .map_err(|e| MyError::from(e))?;

        info!("Delete list id={:?}", row.id);

        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::error::MyError;

    const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

    async fn list_count(pool: PgPool) -> Result<i64, MyError> {
        let (list_count,): (i64,) = sqlx::query_as("SELECT count(*) FROM lists")
            .fetch_one(&pool)
            .await
            .map_err(|e| MyError::from(e))?;
        Ok(list_count)
    }

    #[test]
    fn lists_serdes() {
        let in_value = json!({
            "name": "pear",
        });

        let my_list: List = serde_json::from_value(in_value).unwrap();

        assert!(my_list.name == "pear");
        assert!(my_list.id == None && my_list.active == None);

        let out_str = serde_json::to_string(&my_list).unwrap();

        assert!(out_str == r#"{"id":null,"name":"pear","active":null}"#);
    }

    // #[sqlx::test]
    // // async fn insert_and_remove_lists(pool: PgPool) -> sqlx::Result<()> {
    // async fn insert_and_remove_lists(pool: PgPool) -> Result<(), Rejection> {
    //     let mut created_ids = vec![];

    //     let example0_unsaved = List::new("example0");

    //     assert_eq!(0, list_count(pool.clone()).await?);

    //     // Create a List
    //     let res = handlers::create(example0_unsaved.clone(), pool.clone())
    //         .await
    //         .expect("Record Created");
    //     assert!(res.name == example0_unsaved.name);
    //     assert!(res.id.is_some());
    //     assert_eq!(1, list_count(pool.clone()).await?);
    //     created_ids.push(res.id.unwrap());

    //     let example0_saved = res;

    //     // Read back the List
    //     let res = handlers::read(example0_saved.id.unwrap(), pool.clone())
    //         .await
    //         .expect("Read back List");
    //     assert_eq!(example0_saved, res);

    //     // Fail create of a List
    //     let res = handlers::create(example0_unsaved.clone(), pool.clone())
    //         .await
    //         .expect_err("Create Rejected");
    //     assert!(matches!(
    //         res.find::<MyError>().unwrap(),
    //         MyError::SqlxError { .. }
    //     ));
    //     assert_eq!(1, list_count(pool.clone()).await?);

    //     // Create a List
    //     let example1_unsaved = List::new("example1");
    //     let res = handlers::create(example1_unsaved.clone(), pool.clone())
    //         .await
    //         .expect("Record Created");
    //     assert_eq!(2, list_count(pool.clone()).await?);
    //     created_ids.push(res.id.unwrap());

    //     // Check the list is returned and matches sorted
    //     let listopts = ListOptions {
    //         limit: None,
    //         offset: None,
    //     };
    //     let res = handlers::list(listopts, pool.clone())
    //         .await
    //         .expect("Got ids of records");

    //     created_ids.sort();
    //     // Do not need to sort res.ids as they are expected to be sorted on return
    //     assert_eq!(res.ids, created_ids);

    //     // Update operation although no actual change
    //     let res = handlers::update(
    //         example0_saved.id.unwrap(),
    //         example0_saved.clone(),
    //         pool.clone(),
    //     )
    //     .await
    //     .expect("Update record");
    //     assert_eq!(res, example0_saved);

    //     // Delete List
    //     let res = handlers::delete(example0_saved.id.unwrap(), pool.clone())
    //         .await
    //         .expect("Deleted List");

    //     let mut unused_id = 0;
    //     while created_ids.contains(&unused_id) {
    //         unused_id += 1;
    //     }

    //     // Delete a List id that does not exist and expect a fail
    //     let _res = handlers::delete(unused_id, pool.clone())
    //         .await
    //         .expect_err("Fail as no List item to delete");

    //     // Read a non-existant id and expect a fail
    //     let _res = handlers::read(unused_id, pool.clone())
    //         .await
    //         .expect_err("Fail a no List item to read");

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_filters_matching() -> Result<(), MyError> {
    //     println!("TODO: Test here");
    //     Ok(())
    // }

    // #[sqlx::test]
    // async fn list_create(pool: PgPool) -> Result<(), Rejection> {
    //     println!("TODO: Test edge cases for each handler");
    //     Ok(())
    // }

    // #[sqlx::test]
    // async fn insert_and_remove_lists_with_filters(pool: PgPool) -> Result<(), Rejection> {
    //     let filter_create = warp::body::json::<List>()
    //         .and(with_db_pool_pg(pool.clone()))
    //         .and_then(handlers::create);

    //     let example0_unsaved = List {
    //         id: None,
    //         name: "example0".to_string(),
    //         active: None,
    //     };

    //     // Try to insert record and shall receive saved record (ie id is provided in response)
    //     let req = warp::test::request().json(&example0_unsaved);

    //     let res1 = req.filter(&filter_create).await?;
    //     assert!(res1.name == example0_unsaved.name);
    //     assert!(res1.id.is_some());

    //     assert_eq!(1, list_count(pool.clone()).await?);

    //     // Try to insert same records and shall receive a SqlxError
    //     let req = warp::test::request().json(&example0_unsaved);
    //     let rej2 = req.filter(&filter_create).await.err().unwrap();

    //     assert!(matches!(
    //         rej2.find::<MyError>().unwrap(),
    //         MyError::SqlxError { .. }
    //     ));

    //     assert_eq!(1, list_count(pool.clone()).await?);

    //     let filter_delete = warp::path::param()
    //         .and(with_db_pool_pg(pool.clone()))
    //         .and_then(handlers::delete);

    //     // Delete the record we just inserted
    //     let req = warp::test::request().path(&format!("/{}", res1.id.unwrap()));

    //     let res3 = req.filter(&filter_delete).await?;

    //     assert_eq!(res3, res1);
    //     assert_eq!(0, list_count(pool.clone()).await?);

    //     // Attempt to delete the same record again and get error reply
    //     let req = warp::test::request().path(&format!("/{}", res1.id.unwrap()));

    //     let rej4 = req.filter(&filter_delete).await.err().unwrap();
    //     assert!(matches!(
    //         rej4.find::<MyError>().unwrap(),
    //         MyError::SqlxError { .. }
    //     ));

    //     let res = handlers::create(example0_unsaved.clone(), pool.clone())
    //         .await
    //         .expect("Record Created");
    //     assert!(res.name == example0_unsaved.name);
    //     assert!(res.id.is_some());

    //     Ok(())
    // }
}
