

#[derive(Debug)]
struct StationSetup {
    setup_id: Option<i32>,
    contact_id: i32,
    radio_model: Option<String>,
    antenna_type: Option<String>,
    power_output: Option<f64>,
    other_equipment: Option<String>,
}
