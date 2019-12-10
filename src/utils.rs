use chrono::{DateTime, Utc};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::SystemTime;

pub fn hash<T: Hash>(value: &T) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish().to_string()
}

pub fn create_ts_directory(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let system_time = SystemTime::now();
    let date_time: DateTime<Utc> = system_time.into();
    let ts = date_time.format("%Y-%m-%d--%H-%M-%S--%z").to_string();

    let path = PathBuf::from(format!("{}/{}", prefix, ts));
    fs::create_dir_all(&path)?;

    Ok(path)
}
