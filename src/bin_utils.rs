use crate::job::BLACKLIST_CONTENT_TYPES;
use crate::lib_utils::create_ts_directory;
use crate::shared;
use crate::traits::{Fetch, Persist};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Default, Clone, Eq, PartialEq, Hash)]
pub struct Fetcher;
impl Fetcher {
    pub fn new() -> Self {
        Fetcher
    }
}
impl Fetch for Fetcher {
    fn get_content_type_blacklist<'a>(&self) -> Option<Vec<&'a str>> {
        let blacklist: Vec<&str> = From::from(&BLACKLIST_CONTENT_TYPES[..]);
        Some(blacklist)
    }
}

pub struct FSPersister {
    out_dir: PathBuf,
}
impl FSPersister {
    pub fn new(root_dir: &str) -> shared::Result<Self> {
        // we need to compute the value for `out_dir` in the `new` method here to ensure
        // that only 1 directory is created when using the `Persister`
        let out_dir = Self::create_out_dir(root_dir)?;
        Ok(FSPersister { out_dir })
    }

    fn create_out_dir(root_dir: &str) -> shared::Result<PathBuf> {
        create_ts_directory(root_dir)
    }
}
impl Persist for FSPersister {
    fn persist(&self, content_id: &str, content: &[u8]) -> shared::Result<usize> {
        let mut out_dir = self.out_dir.clone();
        out_dir.push(content_id);
        let mut full_path = File::create(out_dir).unwrap();
        full_path.write_all(content)?;
        Ok(content.len())
    }
}
