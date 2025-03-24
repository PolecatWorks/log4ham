
use std::fmt;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;
use sqlx::{types::{chrono}, Arguments, Postgres};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::types::Decimal;
use crate::webserver::DbBigSerial;

#[derive(sqlx::Type, Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum Band {
    B20m
}

#[derive(sqlx::Type, Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum Mode {
    Ssb
}


struct SerializeDecimal;
impl serde_with::SerializeAs<Decimal> for SerializeDecimal {
    fn serialize_as<S: Serializer>(source: &Decimal, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&source.to_string())
    }
}
// #[derive(Debug)]
// struct MyDecimal(Decimal);

impl<'de> serde_with::DeserializeAs<'de, Decimal> for SerializeDecimal {
    fn deserialize_as<D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Decimal::from_str_exact(&s).map_err(serde::de::Error::custom)
    }
}





// Define structs to represent our database tables
#[serde_with::serde_as]
#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, Clone, PartialEq)]
pub(crate) struct Contact {
    contact_id: Option<i32>,
    user_id: DbBigSerial,
    qso_date: NaiveDate,
    qso_time: NaiveTime,
    callsign: String,

    operator_callsign: String,
    band: Band,
    // #[serde(skip_serializing, skip_deserializing)]
    // #[serde(serialize_with = "serialize_decimal", deserialize_with = "deserialize_decimal")]
    #[serde_as(as = "Option<SerializeDecimal>")]
    // #[serde(deserialize_with = "deserialize_decimal")]
    frequency: Option<Decimal>,
    mode: Mode,

    rst_sent: Option<String>,
    rst_received: Option<String>,
    name_received: Option<String>,
    qth_received: Option<String>,

    grid_square: Option<String>,
    country: Option<String>,
    state_province: Option<String>,
    county: Option<String>,

    notes: Option<String>,
    is_confirmed: bool,
    // created_at: Option<DateTime<Utc>>,
    // updated_at: Option<DateTime<Utc>>,
}

impl Contact {
    pub fn new(
        contact_id: Option<i32>, user_id: DbBigSerial,
        qso_date: NaiveDate, qso_time: NaiveTime, callsign: String,
        operator_callsign: String, band: Band, frequency: Option<Decimal>, mode: Mode,
        rst_sent: Option<String>, rst_received: Option<String>, name_received: Option<String>, qth_received: Option<String>,
        grid_square: Option<String>, country: Option<String>, state_province: Option<String>, county: Option<String>,
        notes: Option<String>, is_confirmed: bool,
        ) -> Self {
        Self {
            contact_id,
            user_id,
            qso_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            qso_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            callsign,

            operator_callsign,
            band,
            frequency,
            mode,

            rst_sent,
            rst_received,
            name_received,
            qth_received,

            grid_square,
            country,
            state_province,
            county,

            notes,
            is_confirmed,
            // created_at: None,
            // updated_at: None,

        }
    }
}



impl<'q> sqlx::IntoArguments<'q, Postgres> for Contact
{
    fn into_arguments(self) -> <sqlx::Postgres as sqlx::Database>::Arguments<'q> {
        let mut arguments = <sqlx::Postgres as sqlx::Database>::Arguments::default();
        arguments.add(self.contact_id).unwrap();
        arguments.add(self.user_id).unwrap();
        arguments.add(self.qso_date).unwrap();
        arguments.add(self.qso_time).unwrap();
        arguments.add(self.callsign).unwrap();
        arguments.add(self.operator_callsign).unwrap();
        arguments.add(self.band).unwrap();
        // arguments.add(self.frequency).unwrap();
        arguments.add(self.mode).unwrap();
        arguments.add(self.rst_sent).unwrap();
        arguments.add(self.rst_received).unwrap();
        arguments.add(self.name_received).unwrap();
        arguments.add(self.qth_received).unwrap();
        arguments.add(self.grid_square).unwrap();
        arguments.add(self.country).unwrap();
        arguments.add(self.state_province).unwrap();
        arguments.add(self.county).unwrap();
        arguments.add(self.notes).unwrap();
        arguments.add(self.is_confirmed).unwrap();
        // arguments.add(self.qso_date).unwrap();
        arguments
    }
}


#[derive(Debug)]
struct QslCard {
    qsl_id: Option<i32>,
    contact_id: i32,
    qsl_sent_date: Option<NaiveDate>,
    qsl_sent_via: Option<String>,
    qsl_received_date: Option<NaiveDate>,
    qsl_received_via: Option<String>,
    qsl_message: Option<String>,
}

#[derive(Debug)]
struct StationSetup {
    setup_id: Option<i32>,
    contact_id: i32,
    radio_model: Option<String>,
    antenna_type: Option<String>,
    power_output: Option<f64>,
    other_equipment: Option<String>,
}

#[cfg(test)]
mod tests {
    use sqlx::types::Decimal;


    // Test the serialisation of Decimal
    #[test]
    fn test_decimal_serialization() {
        let decimal = sqlx::types::Decimal::new(202, 2);

        let encoded = decimal.serialize();

        let decoded = Decimal::deserialize(encoded);

        assert_eq!(decimal, decoded);
    }

    #[test]
    fn test_decimal_string() {
        let decimal = sqlx::types::Decimal::new(202, 2);

        let encoded = decimal.to_string();

        let decoded = Decimal::from_str_exact(&encoded).unwrap();

        assert_eq!(decimal, decoded);
    }

    /// Test the serialisation of Contact using serde_json
    #[test]
    fn test_contact_serialization() {
        let contact = crate::webserver::logs::structs::Contact::new(
            None, 1,
            chrono::NaiveDate::parse_from_str("2023-01-01", "%Y-%m-%d").unwrap(), chrono::NaiveTime::parse_from_str("12:00", "%H:%M").unwrap(), "CALLSIGN".to_string(),
            "MI7IEU".to_string(), crate::webserver::logs::structs::Band::B20m, Some(Decimal::new(202, 2)), crate::webserver::logs::structs::Mode::Ssb,
            Some("59".to_string()), Some("59".to_string()), Some("John".to_string()), Some("Belfast".to_string()),
            Some("IO64".to_string()), Some("United Kingdom".to_string()), Some("Northern Ireland".to_string()), Some("Antrim".to_string()),
            Some("Some notes".to_string()), true,
        );

        let encoded = serde_json::to_string(&contact).unwrap();

        let decoded: crate::webserver::logs::structs::Contact = serde_json::from_str(&encoded).unwrap();

        assert_eq!(contact, decoded);
        println!("encoded: {}", encoded);
    }

}
