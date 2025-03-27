use sqlx::PgPool;

use crate::{
    error::MyError,
    webserver::{DbBigSerial, DbId, ListPages, PageOptions},
};

use super::QslCard;

pub(crate) async fn list(
    options: PageOptions,
    pool_pg: PgPool,
) -> Result<ListPages, warp::Rejection> {
    let ids = sqlx::query_as::<_, DbId>("SELECT id FROM qsl_cards")
        .fetch_all(&pool_pg)
        .await
        .map_err(MyError::from)?;

    let list_ids = ListPages {
        pagination: options,
        ids: ids.iter().map(|u| u.id).collect(),
    };

    Ok(list_ids)
}

pub(crate) async fn create(pool_pg: PgPool, qsl_card: QslCard) -> Result<QslCard, warp::Rejection> {
    let record = sqlx::query_as_with(
        r#"
        INSERT INTO qsl_cards (contact_id, qsl_sent_date, qsl_sent_via, qsl_received_date, qsl_received_via, qsl_message)
        VALUES ($2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        qsl_card,
    )
    .fetch_one(&pool_pg)
    .await
    .map_err(MyError::from)?;

    Ok(record)
}

pub(crate) async fn read(pool_pg: PgPool, id: DbBigSerial) -> Result<QslCard, warp::Rejection> {
    let record = sqlx::query_as::<_, QslCard>("SELECT * FROM qsl_cards WHERE id = $1")
        .bind(id)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

    Ok(record)
}

pub(crate) async fn update(
    pool_pg: PgPool,
    id: DbBigSerial,
    qsl_card: QslCard,
) -> Result<QslCard, warp::Rejection> {
    if qsl_card.id.is_none() || id != qsl_card.id.unwrap() {
        return Err(MyError::Message("ids on path and body must match for update").into());
    }

    let record = sqlx::query_as_with(
        r#"
        UPDATE qsl_cards
        SET contact_id = $2, qsl_sent_date = $3, qsl_sent_via = $4, qsl_received_date = $5, qsl_received_via = $6, qsl_message = $7
        WHERE id = $1
        RETURNING *
        "#,
        qsl_card,
    )
    .fetch_one(&pool_pg)
    .await
    .map_err(MyError::from)?;

    Ok(record)
}

pub(crate) async fn delete(pool_pg: PgPool, id: DbBigSerial) -> Result<QslCard, warp::Rejection> {
    let record = sqlx::query_as("DELETE FROM qsl_cards WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

    Ok(record)
}

#[cfg(test)]
mod tests {
    use sqlx::types::Decimal;

    use crate::webserver::{
        contacts::{self, qsl::QslCardBuilder, Band, Contact, Mode},
        users,
    };

    use super::*;

    /// Test list returns empty list
    #[sqlx::test]
    async fn test_list_empty(pool: PgPool) -> sqlx::Result<()> {
        let my_list = list(PageOptions::default(), pool.clone()).await.unwrap();

        println!("list: {:?}", my_list);

        assert_eq!(my_list.ids.len(), 0);
        Ok(())
    }

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

    /// Test create a QSL card and confirm it is listed
    #[sqlx::test]
    async fn test_create(pool: PgPool) -> sqlx::Result<()> {
        let contact = create_contact(pool.clone()).await;

        let qsl_card = QslCardBuilder::default()
            .contact_id(contact.id.unwrap())
            .build()
            .unwrap();

        let my_qsl_card = create(pool.clone(), qsl_card).await.unwrap();

        println!("qsl_card: {:?}", my_qsl_card);

        assert_eq!(my_qsl_card.contact_id, 1);
        assert_eq!(my_qsl_card.qsl_sent_date, None);
        assert_eq!(my_qsl_card.qsl_sent_via, None);
        assert_eq!(my_qsl_card.qsl_received_date, None);
        assert_eq!(my_qsl_card.qsl_received_via, None);
        assert_eq!(my_qsl_card.qsl_message, None);

        let my_list = list(PageOptions::default(), pool.clone()).await.unwrap();

        println!("list: {:?}", my_list);

        assert_eq!(my_list.ids.len(), 1);

        let read_qsl_card = read(pool.clone(), my_qsl_card.id.unwrap()).await.unwrap();

        println!("read_qsl_card: {:?}", read_qsl_card);
        assert_eq!(my_qsl_card, read_qsl_card);

        let delete_qsl_card = delete(pool.clone(), my_qsl_card.id.unwrap()).await.unwrap();

        println!("delete_qsl_card: {:?}", delete_qsl_card);
        assert_eq!(my_qsl_card, delete_qsl_card);

        let my_list = list(PageOptions::default(), pool.clone()).await.unwrap();

        println!("list: {:?}", my_list);

        assert_eq!(my_list.ids.len(), 0);

        Ok(())
    }

    /// Test a read on a non-existent QSL card
    #[sqlx::test]
    async fn test_read_non_existent(pool: PgPool) -> sqlx::Result<()> {
        let my_qsl_card = read(pool.clone(), 1).await;

        println!("qsl_card: {:?}", my_qsl_card);

        assert!(my_qsl_card.is_err());

        Ok(())
    }

    /// Test an update on a non-existent QSL card
    #[sqlx::test]
    async fn test_update_non_existent(pool: PgPool) -> sqlx::Result<()> {
        let qsl_card = QslCardBuilder::default().contact_id(1).build().unwrap();

        let my_qsl_card = update(pool.clone(), 1, qsl_card).await;

        println!("qsl_card: {:?}", my_qsl_card);

        assert!(my_qsl_card.is_err());

        Ok(())
    }

    /// Test an delete on a non-existent QSL card
    #[sqlx::test]
    async fn test_delete_non_existent(pool: PgPool) -> sqlx::Result<()> {
        let my_qsl_card = delete(pool.clone(), 1).await;

        println!("qsl_card: {:?}", my_qsl_card);

        assert!(my_qsl_card.is_err());

        Ok(())
    }
}
