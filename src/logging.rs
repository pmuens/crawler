extern crate chrono;

use self::chrono::Utc;
use chrono::DateTime;
use std::time::SystemTime;

pub enum Log {
    Info(String),
    Warn(String),
    Fatal(String),
}

impl Log {
    pub fn print(&self) {
        let system_time = SystemTime::now();
        let date_time: DateTime<Utc> = system_time.into();
        let time_formatted = date_time.format("%d/%m/%Y %T");
        match self {
            Log::Info(msg) => println!("INFO - {} - \"{}\"", time_formatted, msg),
            Log::Warn(msg) => println!("WARN - {} - \"{}\"", time_formatted, msg),
            Log::Fatal(msg) => eprintln!("FATAL - {} - \"{}\"", time_formatted, msg),
        }
    }
}
