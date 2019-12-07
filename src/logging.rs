use chrono::DateTime;
use chrono::Utc;
use std::fmt::{self, Display};
use std::time::SystemTime;

pub enum Log {
    Info(String),
    Warn(String),
    Fatal(String),
}

impl Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let system_time = SystemTime::now();
        let date_time: DateTime<Utc> = system_time.into();
        let time_formatted = date_time.format("%d/%m/%Y %T");
        let msg = match self {
            Log::Info(msg) => format!("INFO - {} - \"{}\"", time_formatted, msg),
            Log::Warn(msg) => format!("WARN - {} - \"{}\"", time_formatted, msg),
            Log::Fatal(msg) => format!("FATAL - {} - \"{}\"", time_formatted, msg),
        };
        write!(f, "{}", msg)
    }
}

#[test]
fn log_types() {
    let info = Log::Info("Foo".to_string()).to_string();
    assert!(info.starts_with("INFO - "));
    let warn = Log::Warn("Foo".to_string()).to_string();
    assert!(warn.starts_with("WARN - "));
    let fatal = Log::Fatal("Foo".to_string()).to_string();
    assert!(fatal.starts_with("FATAL - "));
}
