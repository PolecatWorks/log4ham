use sqlx::PgPool;

use crate::{
    error::MyError,
    webserver::{DbBigSerial, DbId, ListPages, PageOptions},
};

use super::Contact;

pub async fn list(options: PageOptions, pool_pg: PgPool) -> Result<ListPages, warp::Rejection> {
    let ids = sqlx::query_as::<_, DbId>("SELECT id FROM contacts")
        .fetch_all(&pool_pg)
        .await
        .map_err(MyError::from)?;

    let list_ids = ListPages {
        pagination: options,
        ids: ids.iter().map(|u| u.id).collect(),
    };

    Ok(list_ids)
}

pub(crate) async fn create(pool_pg: PgPool, contact: Contact) -> Result<Contact, warp::Rejection> {
    let record = sqlx::query_as_with(
        // rst_sent, rst_received, name_received, qth_received,
        // grid_square, country, state_province, county,
        // notes, is_confirmed

        // $9, $10, $11, $12,
        // $13, $14, $15, $16,
        // $17, $18
        r#"
        INSERT INTO contacts (
            user_id,
            qso_date, qso_time, callsign,
            operator_callsign, band, frequency, mode,
            rst_sent, rst_received, name_received, qth_received,
            grid_square, country, state_province, county,
            notes, is_confirmed

        )
        VALUES ($2, $3, $4, $5,
            $6, $7::band, $8, $9::mode,
            $10, $11, $12, $13,
            $14, $15, $16, $17,
            $18, $19
            )
        RETURNING *
        "#,
        contact,
    )
    .fetch_one(&pool_pg)
    .await
    .map_err(MyError::from)?;

    Ok(record)
}

pub async fn read(pool_pg: PgPool, id: DbBigSerial) -> Result<Contact, warp::Rejection> {
    let log = sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE id = $1")
        .bind(id)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

    Ok(log)
}

pub async fn update(
    pool_pg: PgPool,
    id: DbBigSerial,
    contact: Contact,
) -> Result<Contact, warp::Rejection> {
    if contact.id.is_none() || id != contact.id.unwrap() {
        return Err(MyError::Message("ids on path and body must match for update").into());
    }

    let record = sqlx::query_as_with(
        r#"
        UPDATE contacts
        SET
            user_id = $2,
            qso_date = $3,
            qso_time = $4,
            callsign = $5,
            operator_callsign = $6,
            band = $7::band,
            frequency = $8,
            mode = $9::mode,
            rst_sent = $10,
            rst_received = $11,
            name_received = $12,
            qth_received = $13,
            grid_square = $14,
            country = $15,
            state_province = $16,
            county = $17,
            notes = $18,
            is_confirmed = $19
        WHERE id = $1
        RETURNING *
        "#,
        contact,
    )
    .fetch_one(&pool_pg)
    .await
    .map_err(MyError::from)?;

    Ok(record)
}

pub async fn delete(pool_pg: PgPool, id: DbBigSerial) -> Result<Contact, warp::Rejection> {
    let record = sqlx::query_as::<_, Contact>("DELETE FROM contacts WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_one(&pool_pg)
        .await
        .map_err(MyError::from)?;

    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webserver::{
        contacts::{handlers::create, Band, Contact, Mode},
        users,
    };
    use sqlx::{types::Decimal, PgPool, Row};

    #[sqlx::test(migrations = false)]
    async fn db_connectivity(pool: PgPool) -> sqlx::Result<()> {
        let foo = sqlx::query("SELECT 1").fetch_one(&pool).await?;

        assert_eq!(foo.get::<i32, _>(0), 1);

        Ok(())
    }

    /// Test out the list function
    #[sqlx::test()]
    async fn list_contacts_empty(pool: PgPool) -> sqlx::Result<()> {
        let my_list = list(PageOptions::default(), pool.clone()).await.unwrap();
        println!("list: {:?}", my_list);

        assert_eq!(my_list.ids.len(), 0);
        Ok(())
    }

    /// Test out the create_contact
    #[sqlx::test()]
    async fn create_minimal_contact_test(pool: PgPool) -> sqlx::Result<()> {
        let user =
            users::handlers::create(users::User::new("test", "user", "password"), pool.clone())
                .await
                .unwrap();

        let new_contact = Contact::new(
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
        );
        let created_contact = create(pool.clone(), new_contact).await.unwrap();
        println!("created contact: {:?}", created_contact);

        let my_list = list(PageOptions::default(), pool.clone()).await.unwrap();

        assert_eq!(my_list.ids.len(), 1);

        Ok(())
    }

    /// Test out the read returns error when no user matches
    #[sqlx::test()]
    async fn read_contact_not_found(pool: PgPool) -> sqlx::Result<()> {
        let my_read = read(pool.clone(), 1).await;

        assert!(my_read.is_err());

        Ok(())
    }

    /// Test out the read returns contact when user matches
    #[sqlx::test()]
    async fn read_contact_found(pool: PgPool) -> sqlx::Result<()> {
        let user =
            users::handlers::create(users::User::new("test", "user", "password"), pool.clone())
                .await
                .unwrap();

        let new_contact = Contact::new(
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
        );
        let created_contact = create(pool.clone(), new_contact).await.unwrap();
        println!("created contact: {:?}", created_contact);

        let my_read = read(pool.clone(), created_contact.id.unwrap())
            .await
            .unwrap();
        println!("read contact: {:?}", my_read);

        assert_eq!(my_read.id, created_contact.id);

        Ok(())
    }

    /// Test out the update returns error when no user matches
    #[sqlx::test()]
    async fn update_contact_not_found(pool: PgPool) -> sqlx::Result<()> {
        let user =
            users::handlers::create(users::User::new("test", "user", "password"), pool.clone())
                .await
                .unwrap();

        let new_contact = Contact::new(
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
        );

        let my_update = update(pool.clone(), 1, new_contact).await;

        assert!(my_update.is_err());

        Ok(())
    }

    /// Test out the update returns updated contact when user matches
    #[sqlx::test()]
    async fn update_contact_found(pool: PgPool) -> sqlx::Result<()> {
        let user =
            users::handlers::create(users::User::new("test", "user", "password"), pool.clone())
                .await
                .unwrap();

        let new_contact = Contact::new(
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
        );

        let created_contact = create(pool.clone(), new_contact).await.unwrap();

        let user2 =
            users::handlers::create(users::User::new("test", "user", "password"), pool.clone())
                .await
                .unwrap();

        println!("created contact: {:?}", created_contact);

        let updated_contact = Contact::new(
            created_contact.id,
            user2.id.unwrap(),
            chrono::NaiveDate::parse_from_str("2023-01-02", "%Y-%m-%d").unwrap(),
            chrono::NaiveTime::parse_from_str("12:01", "%H:%M").unwrap(),
            "CALLSIGN_".to_string(),
            "MI7TBK".to_string(),
            Band::B40m,
            Some(Decimal::new(203, 2)),
            Mode::Cw,
            Some("59_".to_string()),
            Some("59_".to_string()),
            Some("NAME_".to_string()),
            Some("QTH_".to_string()),
            Some("GRID_".to_string()),
            Some("COUNTRY_".to_string()),
            Some("STATE_".to_string()),
            Some("COUNTY_".to_string()),
            Some("NOTES_".to_string()),
            true,
        );

        let my_update = update(
            pool.clone(),
            created_contact.id.unwrap(),
            updated_contact.clone(),
        )
        .await
        .unwrap();
        println!("updated contact: {:?}", my_update);

        assert_eq!(my_update.id, updated_contact.id);

        assert!(my_update.content_eq(&updated_contact));
        assert!(!my_update.content_eq(&created_contact));
        Ok(())
    }

    /// Test out the delete returns error when no user matches
    #[sqlx::test()]
    async fn delete_contact_not_found(pool: PgPool) -> sqlx::Result<()> {
        let my_delete = delete(pool.clone(), 1).await;

        assert!(my_delete.is_err());

        Ok(())
    }

    /// Test out the delete returns deleted contact when user matches
    /// and the contact is deleted
    #[sqlx::test()]
    async fn delete_contact_found(pool: PgPool) -> sqlx::Result<()> {
        let user =
            users::handlers::create(users::User::new("test", "user", "password"), pool.clone())
                .await
                .unwrap();

        let new_contact = Contact::new(
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
        );

        let created_contact = create(pool.clone(), new_contact).await.unwrap();
        println!("created contact: {:?}", created_contact);

        let my_delete = delete(pool.clone(), created_contact.id.unwrap())
            .await
            .unwrap();
        println!("deleted contact: {:?}", my_delete);

        let my_list = list(PageOptions::default(), pool.clone()).await.unwrap();

        assert_eq!(my_list.ids.len(), 0);

        Ok(())
    }
}
