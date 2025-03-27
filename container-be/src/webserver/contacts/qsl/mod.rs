mod handlers;

use chrono::NaiveDate;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use sqlx::{Arguments, Postgres};

use crate::webserver::DbBigSerial;

/// A QSL card is a written confirmation of a two-way radio communication between two amateur radio operators.
/// This struct represents a QSL card in the database.
#[derive(Deserialize, Serialize, Default, Debug, sqlx::FromRow, Clone, PartialEq, Builder)]
#[builder(default)]
pub(crate) struct QslCard {
    /// Unique identifier for the QSL card
    id: Option<DbBigSerial>,

    /// Unique identifier for the contact
    contact_id: DbBigSerial,
    /// The date the QSL card was sent
    qsl_sent_date: Option<NaiveDate>,
    /// The method by which the QSL card was sent
    qsl_sent_via: Option<String>,
    /// The date the QSL card was received
    qsl_received_date: Option<NaiveDate>,
    /// The method by which the QSL card was received
    qsl_received_via: Option<String>,
    /// The message included with the QSL card
    qsl_message: Option<String>,
}

/// Define a sqlx::IntoArguments implementation for QslCard so that we can use it in SQL queries.
impl<'q> sqlx::IntoArguments<'q, Postgres> for QslCard {
    fn into_arguments(self) -> <Postgres as sqlx::Database>::Arguments<'q> {
        let mut arguments = <Postgres as sqlx::Database>::Arguments::default();

        arguments.add(self.id).unwrap();

        arguments.add(self.contact_id).unwrap();
        arguments.add(self.qsl_sent_date).unwrap();
        arguments.add(self.qsl_sent_via).unwrap();
        arguments.add(self.qsl_received_date).unwrap();
        arguments.add(self.qsl_received_via).unwrap();
        arguments.add(self.qsl_message).unwrap();

        arguments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qsl_card_builder() {
        let qsl_card = QslCardBuilder::default()
            .id(Some(1))
            .contact_id(1)
            .qsl_sent_date(Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()))
            .qsl_sent_via(Some("Bureau".to_string()))
            .qsl_received_date(Some(NaiveDate::from_ymd_opt(2021, 1, 2).unwrap()))
            .qsl_received_via(Some("Direct".to_string()))
            .qsl_message(Some("Thanks for the QSL card!".to_string()))
            .build()
            .unwrap();

        assert_eq!(qsl_card.contact_id, 1);
        assert_eq!(qsl_card.qsl_sent_date, NaiveDate::from_ymd_opt(2021, 1, 1));
        assert_eq!(qsl_card.qsl_sent_via, Some("Bureau".to_string()));
        assert_eq!(
            qsl_card.qsl_received_date,
            NaiveDate::from_ymd_opt(2021, 1, 2)
        );
        assert_eq!(qsl_card.qsl_received_via, Some("Direct".to_string()));
        assert_eq!(
            qsl_card.qsl_message,
            Some("Thanks for the QSL card!".to_string())
        );
    }

    /// Test the QslCard with a minimal set of fields via the builder
    #[test]
    fn test_qsl_card_minimal() {
        let qsl_card = QslCardBuilder::default().contact_id(1).build().unwrap();

        assert_eq!(qsl_card.contact_id, 1);
        assert_eq!(qsl_card.qsl_sent_date, None);
        assert_eq!(qsl_card.qsl_sent_via, None);
        assert_eq!(qsl_card.qsl_received_date, None);
        assert_eq!(qsl_card.qsl_received_via, None);
        assert_eq!(qsl_card.qsl_message, None);
    }
}
