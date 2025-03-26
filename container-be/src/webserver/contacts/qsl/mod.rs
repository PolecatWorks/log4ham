use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::webserver::DbBigSerial;

/// A QSL card is a written confirmation of a two-way radio communication between two amateur radio operators.
/// This struct represents a QSL card in the database.
#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, Clone, PartialEq)]
struct QslCard {
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
