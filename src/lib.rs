#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate regex;
extern crate reqwest;

pub mod args;
pub mod bin_utils;
pub mod crawling;
pub mod job;
pub mod lib_utils;
pub mod traits;
#[macro_use]
mod logging;

use crawling::Crawling;
use job::{Job, Queue};
use lib_utils::CrawlingResult;
use reqwest::Url;
use std::error::Error;
use std::hash::Hash;
use std::sync::Arc;
use std::thread;
use traits::{Fetch, Persist};

// TODO: make this configurable from the outside
static QUEUE_BUFFER: usize = 1_000_000;

pub struct Crawler<A, B>
where
    A: Persist,
    B: Fetch,
{
    queue: Queue<Job<B>>,
    num_threads: usize,
    persister: Arc<A>,
    fetcher: Arc<B>,
}

impl<A: 'static, B: 'static> Crawler<A, B>
where
    A: Persist + Send + Sync,
    B: Fetch + Eq + Clone + Hash + Send + Sync,
{
    pub fn new(persister: A, fetcher: B, num_threads: usize) -> Self {
        Crawler {
            queue: Queue::new(QUEUE_BUFFER),
            persister: Arc::new(persister),
            fetcher: Arc::new(fetcher),
            num_threads,
        }
    }

    pub fn start(&mut self, url: &str) -> Result<(), Box<dyn Error>> {
        let url = Url::parse(url)?;
        let initial_job = Job::new(self.fetcher.clone(), url).unwrap();
        self.queue.enqueue(initial_job);

        loop {
            let mut handlers = Vec::with_capacity(self.num_threads);
            for _ in 0..self.num_threads {
                if let Some(job) = self.queue.dequeue() {
                    let persister = self.persister.clone();
                    let fetcher = self.fetcher.clone();
                    let handler = thread::spawn(move || {
                        if let Ok(result) = crawl(persister, fetcher, job) {
                            result.crawling.write().unwrap_or_else(|_| 0);
                            return result.jobs;
                        }
                        None
                    });
                    handlers.push(handler);
                }
            }

            for handler in handlers {
                if let Some(jobs) = handler.join().unwrap() {
                    for job in jobs {
                        self.queue.enqueue(job);
                    }
                }
            }

            if self.queue.is_empty() {
                break;
            }
        }

        Ok(())
    }
}

fn crawl<A, B>(
    persister: Arc<A>,
    fetcher: Arc<B>,
    job: Job<B>,
) -> Result<CrawlingResult<A, B>, Box<dyn Error>>
where
    A: Persist,
    B: Fetch,
{
    let url = job.get_url();
    log!(format!("GET {}", &url));
    if let Ok((content_type, content)) = job.fetch() {
        let crawling = Crawling::new(persister, job.get_url(), content_type.as_str(), content);
        if let Some(urls) = crawling.find_urls() {
            let jobs_arr = Vec::<Job<B>>::with_capacity(urls.len());
            let jobs = urls.into_iter().fold(jobs_arr, |mut accum, url| {
                if let Some(job) = Job::new(fetcher.clone(), url) {
                    accum.push(job);
                }
                accum
            });
            return Ok(CrawlingResult {
                crawling,
                jobs: Some(jobs),
            });
        }
        return Ok(CrawlingResult {
            crawling,
            jobs: None,
        });
    }
    Err(Box::from(format!("Failed to GET {}", &url)))
}
