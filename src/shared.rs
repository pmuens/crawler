use crate::crawling::Crawling;
use crate::error::CrawlerError;
use crate::job::Job;
use crate::job::BLACKLIST_CONTENT_TYPES;
use crate::traits::{Fetch, Persist};
use chrono::{DateTime, Utc};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Default, Clone, Eq, PartialEq, Hash)]
pub struct MainFetcher;
impl MainFetcher {
    pub fn new() -> Self {
        MainFetcher
    }
}
impl Fetch for MainFetcher {
    fn get_content_type_blacklist<'a>(&self) -> Option<Vec<&'a str>> {
        let blacklist: Vec<&str> = From::from(&BLACKLIST_CONTENT_TYPES[..]);
        Some(blacklist)
    }
}

pub struct FSPersister {
    out_dir: PathBuf,
}
impl FSPersister {
    pub fn new(root_dir: &str) -> self::Result<Self> {
        // we need to compute the value for `out_dir` in the `new` method here to ensure
        // that only 1 directory is created when using the `Persister`
        let out_dir = Self::create_out_dir(root_dir)?;
        Ok(FSPersister { out_dir })
    }

    fn create_out_dir(root_dir: &str) -> self::Result<PathBuf> {
        create_ts_directory(root_dir)
    }
}
impl Persist for FSPersister {
    fn persist(&self, id: &str, _url: &str, content: &[u8]) -> self::Result<usize> {
        let mut out_dir = self.out_dir.clone();
        out_dir.push(id);
        let mut full_path = File::create(out_dir).unwrap();
        full_path.write_all(content)?;
        Ok(content.len())
    }
}

pub fn hash<T>(value: &T) -> String
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish().to_string()
}

pub fn create_ts_directory(prefix: &str) -> self::Result<PathBuf> {
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

pub type Result<T> = std::result::Result<T, CrawlerError>;
