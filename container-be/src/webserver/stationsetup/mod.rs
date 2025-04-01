use crate::webserver::SerializeDecimal;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::{types::Decimal, Arguments, Postgres};

use crate::webserver::DbBigSerial;

mod handlers;
pub(crate) mod routes;

/// A station setup is a configuration of equipment used for amateur radio operations.
/// This struct represents a station setup in the database.
/// It includes information about the radio model, antenna type, power output, and other equipment used.
/// This struct is used to store and retrieve station setup information from the database.
#[serde_with::serde_as]
#[derive(Deserialize, Serialize, Default, Debug, sqlx::FromRow, Clone, PartialEq, Builder)]
#[builder(default)]
struct StationSetup {
    /// Unique identifier for the station setup
    id: Option<DbBigSerial>,
    /// Unique identifier for the contact
    contact_id: DbBigSerial,
    /// The date the station setup was created
    radio_model: Option<String>,
    /// The model of the radio used
    antenna_type: Option<String>,
    /// The type of antenna used
    #[serde_as(as = "Option<SerializeDecimal>")]
    power_output: Option<Decimal>,
    /// The power output of the radio in watts
    other_equipment: Option<String>,
}

/// Define a sqlx::IntoArguments implementation for StationSetup so that we can use it in SQL queries.
impl<'q> sqlx::IntoArguments<'q, sqlx::Postgres> for StationSetup {
    fn into_arguments(self) -> <sqlx::Postgres as sqlx::Database>::Arguments<'q> {
        let mut arguments = <sqlx::Postgres as sqlx::Database>::Arguments::default();

        arguments.add(self.id).unwrap();
        arguments.add(self.contact_id).unwrap();
        arguments.add(self.radio_model).unwrap();
        arguments.add(self.antenna_type).unwrap();
        arguments.add(self.power_output).unwrap();
        arguments.add(self.other_equipment).unwrap();

        arguments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_station_setup_builder() {
        let station_setup = StationSetupBuilder::default()
            .id(Some(1))
            .contact_id(1)
            .radio_model(Some("IC-7300".to_string()))
            .antenna_type(Some("Dipole".to_string()))
            .power_output(Some(Decimal::new(100, 0)))
            .other_equipment(Some("None".to_string()))
            .build()
            .unwrap();

        assert_eq!(station_setup.id, Some(1));
        assert_eq!(station_setup.contact_id, 1);
        assert_eq!(station_setup.radio_model, Some("IC-7300".to_string()));
        assert_eq!(station_setup.antenna_type, Some("Dipole".to_string()));
        assert_eq!(station_setup.power_output, Some(Decimal::new(100, 0)));
        assert_eq!(station_setup.other_equipment, Some("None".to_string()));
    }

    /// Test the minimum required fields for the StationSetup struct.
    #[test]
    fn test_station_setup_minimum_fields() {
        let station_setup = StationSetupBuilder::default()
            .contact_id(1)
            .build()
            .unwrap();

        assert_eq!(station_setup.id, None);
        assert_eq!(station_setup.contact_id, 1);
        assert_eq!(station_setup.radio_model, None);
        assert_eq!(station_setup.antenna_type, None);
        assert_eq!(station_setup.power_output, None);
        assert_eq!(station_setup.other_equipment, None);
    }

    /// Test StationSetup serialization and deserialization
    #[test]
    fn test_station_setup_serialization() {
        let station_setup = StationSetupBuilder::default()
            .id(Some(1))
            .contact_id(1)
            .radio_model(Some("IC-7300".to_string()))
            .antenna_type(Some("Dipole".to_string()))
            .power_output(Some(Decimal::new(100, 0)))
            .other_equipment(Some("None".to_string()))
            .build()
            .unwrap();

        let serialized = serde_json::to_string(&station_setup).unwrap();
        let deserialized: StationSetup = serde_json::from_str(&serialized).unwrap();

        assert_eq!(station_setup, deserialized);
    }
}
