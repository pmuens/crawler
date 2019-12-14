use chrono::DateTime;
use chrono::Utc;
use std::time::SystemTime;

pub fn formatted_now() -> String {
    let system_time = SystemTime::now();
    let date_time: DateTime<Utc> = system_time.into();
    date_time.format("%d/%m/%Y %T").to_string()
}

macro_rules! log {
    ($msg:expr) => {
        println!("INFO - {} - \"{}\"", $crate::logging::formatted_now(), $msg);
    };
}

macro_rules! loge {
    ($msg:expr) => {
        eprintln!(
            "FATAL - {} - \"{}\"",
            $crate::logging::formatted_now(),
            $msg
        );
    };
}
