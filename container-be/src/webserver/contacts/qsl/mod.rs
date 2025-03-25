

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
