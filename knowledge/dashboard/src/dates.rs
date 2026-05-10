use chrono::{DateTime, Days, Utc};

pub fn seven_days_ago() -> DateTime<Utc> {
    now().checked_sub_days(Days::new(7)).unwrap()
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}
