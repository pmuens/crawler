use crate::crawling::Crawling;
use crate::job::Job;
use crate::shared;
use crate::traits::{Fetch, Persist};
use chrono::{DateTime, Utc};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::SystemTime;

pub fn hash<T>(value: &T) -> String
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish().to_string()
}

pub fn create_ts_directory(prefix: &str) -> shared::Result<PathBuf> {
    let system_time = SystemTime::now();
    let date_time: DateTime<Utc> = system_time.into();
    let ts = date_time.format("%Y-%m-%d--%H-%M-%S--%z").to_string();

    let path = PathBuf::from(format!("{}/{}", prefix, ts));
    fs::create_dir_all(&path)?;

    Ok(path)
}

pub struct CrawlingResult<A, B>
where
    A: Persist,
    B: Fetch,
{
    pub crawling: Crawling<A>,
    pub jobs: Option<Vec<Job<B>>>,
}
