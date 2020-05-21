use chrono::Local;

pub use log::{debug, info};

pub fn get_now() -> i64 {
    Local::now().naive_utc().timestamp_millis()
}
