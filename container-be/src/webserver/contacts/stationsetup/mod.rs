use crate::webserver::DbBigSerial;

#[derive(Debug)]
struct StationSetup {
    setup_id: Option<DbBigSerial>,
    contact_id: DbBigSerial,
    radio_model: Option<String>,
    antenna_type: Option<String>,
    power_output: Option<f64>,
    other_equipment: Option<String>,
}
