#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate regex;
extern crate reqwest;

mod crawling;
mod job;
mod utils;
#[macro_use]
mod logging;
pub mod args;

use crawling::Crawling;
use job::{Job, JobQueue};
use reqwest::Url;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use utils::{create_ts_directory, hash};

pub fn run_single_threaded(url: &str, out_dir: &str) -> Result<(), Box<dyn Error>> {
    let mut queue = JobQueue::new(1_000_000);
    let dest = create_ts_directory(out_dir)?;

    let url = Url::parse(url)?;
    queue.enqueue(Job::new(url));

    while let Some(job) = queue.dequeue() {
        let crawling = crawl(&job);
        if let Some(crawling) = crawling {
            let urls = crawling.find_urls();
            if let Some(urls) = urls {
                urls.into_iter()
                    .for_each(|url| queue.enqueue(Job::new(url)));
            }
            write_to_disk(&dest, &crawling)?;
        }
    }

    Ok(())
}

fn crawl(job: &Job) -> Option<Crawling> {
    log!(format!("GET {}", job.get_url()));
    let result = job.fetch();

    match result {
        Ok(content) => Some(Crawling::new(job.get_url(), content)),
        Err(err) => {
            loge!(format!("Failed to GET \"{}\": {}", job.get_url(), err));
            None
        }
    }
}

fn write_to_disk(dest: &PathBuf, crawling: &Crawling) -> Result<(), Box<dyn Error>> {
    let file_name = hash(&crawling);
    if let Some(domain_prefix) = crawling.get_domain() {
        let mut new_dest = dest.clone();
        new_dest.push(format!("{}-{}", domain_prefix, file_name));
        let mut full_path = File::create(new_dest)?;
        crawling.write(&mut full_path)?;
        return Ok(());
    }
    Err(Box::from(format!("Error creating file \"{}\"", file_name)))
}
