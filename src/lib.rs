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
use crossbeam_channel::{bounded, Receiver, Sender};
use job::{Job, JobQueue};
use reqwest::Url;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use utils::{create_ts_directory, hash};

// TODO: make this configurable from the outside
static JOB_QUEUE_BUFFER: usize = 1_000_000;
static CHANNEL_CAPACITY: usize = 5_000;

pub fn run(url: &str, out_dir: &str, num_threads: usize) -> Result<(), Box<dyn Error>> {
    let dest = create_ts_directory(out_dir)?;
    let mut queue = JobQueue::new(JOB_QUEUE_BUFFER);
    // we halve the number of threads since we're running 2 separate thread-based workers
    let num_threads = num_threads / 2;

    let (to_enqueue, to_enqueue_recv) = bounded(CHANNEL_CAPACITY);
    let (to_crawl, to_crawl_recv) = bounded(CHANNEL_CAPACITY);
    let (to_persist, to_persist_recv) = bounded(CHANNEL_CAPACITY);

    let h1 = crawling_threads(num_threads, to_crawl_recv, to_enqueue.clone(), to_persist);
    let h2 = persisting_threads(num_threads, to_persist_recv, dest.clone());

    // send initial job over the channel
    let url = Url::parse(url)?;
    to_enqueue.send(Job::new(url).unwrap())?;

    for new_job in to_enqueue_recv {
        queue.enqueue(new_job);
        for _ in 0..num_threads {
            if let Some(job) = queue.dequeue() {
                if to_crawl.send(job).is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    }

    let _r1 = h1.into_iter().map(|h| h.join());
    let _r2 = h2.into_iter().map(|h| h.join());

    Ok(())
}

fn crawling_threads(
    num_threads: usize,
    to_crawl_recv: Receiver<Job>,
    to_enqueue: Sender<Job>,
    to_persist: Sender<Crawling>,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::with_capacity(num_threads);
    for _ in 0..num_threads {
        let to_enqueue = to_enqueue.clone();
        let to_persist = to_persist.clone();
        let to_crawl_recv = to_crawl_recv.clone();
        let handle = thread::spawn(move || {
            for job in to_crawl_recv {
                if let Ok(crawling) = crawl(&job) {
                    // find new urls, turn them into `Job`s and send request to enqueue them
                    crawling.find_urls().and_then(|urls| {
                        let jobs: Vec<Job> = urls.into_iter().filter_map(Job::new).collect();
                        for job in jobs {
                            if to_enqueue.send(job).is_err() {
                                break;
                            }
                        }
                        Some(())
                    });
                    // send request to persist `Crawling`
                    if to_persist.send(crawling).is_err() {
                        break;
                    }
                }
            }
        });
        handles.push(handle);
    }
    handles
}

fn persisting_threads(
    num_threads: usize,
    to_persist_recv: Receiver<Crawling>,
    out_dir: PathBuf,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::with_capacity(num_threads);
    for _ in 0..num_threads {
        let to_persist_recv = to_persist_recv.clone();
        let out_dir = out_dir.clone();
        let handle = thread::spawn(move || {
            for crawling in to_persist_recv {
                write_to_disk(out_dir.clone(), &crawling);
            }
        });
        handles.push(handle);
    }
    handles
}

fn crawl(job: &Job) -> Result<Crawling, Box<dyn Error>> {
    log!(format!("GET {}", job.get_url()));
    let (content_type, content) = job.fetch()?;
    Ok(Crawling::new(job.get_url(), content_type.as_str(), content))
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
