use chrono::Local;

pub fn get_now() -> i64 {
    Local::now().naive_utc().timestamp_millis()
}
