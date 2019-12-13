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
    queue.enqueue(vec![Job::new(url).unwrap()]);

    while let Some(mut jobs) = queue.dequeue(1) {
        let job = jobs.pop().unwrap();
        let crawling = crawl(&job)?;
        let urls = crawling.find_urls();
        if let Some(urls) = urls {
            urls.into_iter()
                .filter_map(Job::new)
                .for_each(|job| queue.enqueue(vec![job]));
        }
        write_to_disk(dest.clone(), &crawling);
    }

    Ok(())
}

fn crawl(job: &Job) -> Result<Crawling, Box<dyn Error>> {
    log!(format!("GET {}", job.get_url()));
    let content = job.fetch()?;
    Ok(Crawling::new(job.get_url(), content))
}

fn write_to_disk(mut dest: PathBuf, crawling: &Crawling) {
    let file_name = hash(&crawling);
    if let Some(domain_prefix) = crawling.get_domain() {
        dest.push(format!("{}-{}", domain_prefix, file_name));
        let mut full_path = File::create(dest).unwrap();
        crawling.write(&mut full_path).unwrap_or_else(|_| {
            loge!(format!("Error creating file \"{}\"", file_name));
        });
    }
}
