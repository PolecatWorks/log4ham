use crate::{
    error::MyError,
    webserver::{DbBigSerial, DbId, ListPages, PageOptions},
};

use super::Log;
use sqlx::PgPool;

pub async fn list(options: PageOptions, pool_pg: PgPool) -> Result<ListPages, warp::Rejection> {
    let ids = sqlx::query_as::<_, DbId>("SELECT id FROM logs")
        .fetch_all(&pool_pg)
        .await
        .map_err(MyError::from)?;

    let list_ids = ListPages {
        pagination: options,
        ids: ids.iter().map(|u| u.id).collect(),
    };

    Ok(list_ids)
}

pub async fn create(new_log: Log, pool_pg: PgPool) -> Result<Log, warp::Rejection> {
    let log = sqlx::query_as::<_, Log>(
        "INSERT INTO logs (user_id, description) VALUES ($1, $2) RETURNING *",
    )
    .bind(new_log.user_id)
    .bind(new_log.description)
    .fetch_one(&pool_pg)
    .await
    .map_err(MyError::from)?;

    Ok(log)
}

pub async fn read(id: DbBigSerial, pool_pg: PgPool) -> Result<Log, warp::Rejection> {
    let log = sqlx::query_as::<_, Log>("SELECT * FROM logs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

    Ok(log)
}

pub async fn update(id: DbBigSerial, log: Log, pool_pg: PgPool) -> Result<Log, warp::Rejection> {
    if log.id.is_none() || id != log.id.unwrap() {
        return Err(MyError::Message("ids on path and body must match for update").into());
    }

    let log = sqlx::query_as::<_, Log>(
        "UPDATE logs SET user_id = $2, description = $3 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(log.user_id)
    .bind(log.description)
    .fetch_one(&pool_pg)
    .await
    .map_err(MyError::from)?;

    Ok(log)
}

pub async fn delete(id: DbBigSerial, pool_pg: PgPool) -> Result<Log, warp::Rejection> {
    let log = sqlx::query_as::<_, Log>("DELETE FROM logs WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

    Ok(log)
}
