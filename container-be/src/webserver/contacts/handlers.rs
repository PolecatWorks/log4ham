use sqlx::PgPool;

use crate::error::MyError;

use super::Contact;




async fn create_contact(pool_pg: PgPool, contact: Contact) -> Result<Contact, warp::Rejection> {
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
            operator_callsign, band, frequency, mode

        )
        VALUES ($2, $3, $4, $5,
            $6, $7::band, $8, $9::mode
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

#[cfg(test)]
mod tests {
    use crate::webserver::{
        contacts::{handlers::create_contact, Band, Contact, Mode}, users
    };
    use sqlx::{types::Decimal, PgPool, Row};

    #[sqlx::test(migrations = false)]
    async fn db_connectivity(pool: PgPool) -> sqlx::Result<()> {
        let foo = sqlx::query("SELECT 1").fetch_one(&pool).await?;

        assert_eq!(foo.get::<i32, _>(0), 1);

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
        let created_contact = create_contact(pool, new_contact).await.unwrap();
        println!("created contact: {:?}", created_contact);
        Ok(())
    }
}
