use reqwest::Client;
use reqwest::Url;
use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::ops::Range;

lazy_static! {
    static ref CLIENT: Client = Client::new();
    static ref BLACKLIST_EXTENSIONS: [&'static str; 14] = [
        ".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".tiff", ".ico", ".json", ".woff2",
        ".csv", ".xls", ".xlsx",
        // NOTE: we might want to re-include the following extensions in future releases
        ".xml"
    ];
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Job {
    url: Url,
}

impl Job {
    pub fn new(url: Url) -> Option<Self> {
        let url_str = url.as_str();
        if BLACKLIST_EXTENSIONS
            .iter()
            .any(|&ext| url_str.ends_with(ext))
        {
            return None;
        }
        Some(Job { url })
    }

    pub fn get_url(&self) -> Url {
        self.url.to_owned()
    }

    pub fn fetch(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut resp = CLIENT.get(&self.url.to_string()).send()?;
        if !resp.status().is_success() {
            Err(resp.status().to_string())?;
        }

        let mut buffer: Vec<u8> = vec![];
        resp.copy_to(&mut buffer)?;

        Ok(buffer)
    }
}

#[test]
fn job() {
    let to_url = |url: &str| Url::parse(url).unwrap();

    // test the common cases
    assert!(Job::new(to_url("http://exampe.com/index")).is_some());
    assert!(Job::new(to_url("http://exampe.com/index.php")).is_some());
    assert!(Job::new(to_url("http://example.com/index.html")).is_some());
    assert!(Job::new(to_url("http://example.com/index.pdf")).is_some());
    assert!(Job::new(to_url("http://example.com/index.ps")).is_some());
    assert!(Job::new(to_url("http://example.com/index.txt")).is_some());
    // test against the blacklist
    for ext in BLACKLIST_EXTENSIONS.iter() {
        let formatted = format!("http://example.com/blacklisted.{}", ext);
        let url = formatted.as_str();
        assert!(Job::new(to_url(url)).is_none());
    }
}

#[derive(Debug)]
pub struct JobQueue {
    queue: VecDeque<Job>,
    visited: HashSet<Job>,
    buffer: usize,
}

impl JobQueue {
    pub fn new(buffer: usize) -> Self {
        JobQueue {
            queue: VecDeque::<Job>::new(),
            visited: HashSet::<Job>::new(),
            buffer,
        }
    }

    // TODO: there might be a more efficient way to do this
    pub fn enqueue(&mut self, jobs: Vec<Job>) {
        // remove the oldest n elements if the queue exceeds the buffer size
        if self.queue.len() + jobs.len() >= self.buffer {
            let range = Range {
                start: 0,
                end: jobs.len(),
            };
            for _ in range {
                self.queue.pop_front();
            }
        }
        // enqueue the new jobs
        for job in jobs {
            if !self.visited.contains(&job) && !self.queue.contains(&job) {
                self.queue.push_back(job);
            }
        }
    }

    // TODO: there might be a more efficient way to do this
    pub fn dequeue(&mut self, count: usize) -> Option<Vec<Job>> {
        // clear the whole set if we exceed the buffer size
        if self.visited.len() + count >= self.buffer {
            self.visited.clear();
        }
        // dequeue the jobs
        let mut jobs = vec![];
        for _ in 0..count {
            if let Some(job) = self.queue.pop_front() {
                // mark as visited when dequeueing it
                self.visited.insert(job.clone());
                jobs.push(job);
            }
        }
        if !jobs.is_empty() {
            return Some(jobs);
        }
        None
    }
}

#[test]
fn job_queue() {
    let to_job = |url: &str| Job::new(Url::parse(url).unwrap()).unwrap();
    let mut q = JobQueue::new(10);

    let job_1 = to_job("http://example.com/1");
    let job_2 = to_job("http://example.com/1");
    let job_3 = to_job("http://example.com/2");
    let job_4 = to_job("http://example.com/3");
    let job_5 = to_job("http://example.com/3");
    let job_6 = to_job("http://example.com/4");
    let job_7 = to_job("http://example.com/4");
    let job_1_c = job_1.clone(); // /1
    let job_3_c = job_3.clone(); // /2
    let job_4_c = job_4.clone(); // /3
    let job_6_c = job_6.clone(); // /4

    q.enqueue(vec![job_1, job_2, job_3]);
    q.enqueue(vec![job_4, job_5, job_6, job_7]);

    assert_eq!(q.dequeue(3), Some(vec![job_1_c, job_3_c, job_4_c]));
    // NOTE: here we're trying to dequeue more items than there are in the queue
    assert_eq!(q.dequeue(4), Some(vec![job_6_c]));
    assert_eq!(q.dequeue(10), None);
    assert_eq!(q.dequeue(1), None);
}

#[test]
fn job_queue_contains() {
    let mut q = JobQueue::new(10);
    let job = Job::new(Url::parse("http://example.com").unwrap()).unwrap();

    q.enqueue(vec![job.clone()]);
    assert!(q.queue.contains(&job));
    assert!(!q.visited.contains(&job));
    q.dequeue(1);
    assert!(!q.queue.contains(&job));
    assert!(q.visited.contains(&job));
    // trying to enqueue the same job again
    q.enqueue(vec![job.clone()]);
    assert!(!q.queue.contains(&job));
    assert!(q.visited.contains(&job));
}

#[test]
fn job_queue_buffer_queue() {
    let to_job = |url: &str| Job::new(Url::parse(url).unwrap()).unwrap();
    let mut q = JobQueue::new(3);

    let job_1 = to_job("http://example.com/1");
    let job_2 = to_job("http://example.com/2");
    let job_3 = to_job("http://example.com/3");
    let job_4 = to_job("http://example.com/4");
    let job_5 = to_job("http://example.com/5");
    let job_3_c = job_3.clone(); // /3
    let job_4_c = job_4.clone(); // /4
    let job_5_c = job_5.clone(); // /5

    q.enqueue(vec![job_1, job_2, job_3]);
    q.enqueue(vec![job_4, job_5]);

    assert_eq!(q.dequeue(2), Some(vec![job_3_c, job_4_c]));
    assert_eq!(q.dequeue(1), Some(vec![job_5_c]));
    assert_eq!(q.dequeue(1), None);
}

#[test]
fn job_queue_buffer_visited() {
    let to_job = |url: &str| Job::new(Url::parse(url).unwrap()).unwrap();
    let mut q = JobQueue::new(2);

    let job_1 = to_job("http://example.com/1");
    let job_2 = to_job("http://example.com/2");
    let job_3 = to_job("http://example.com/3");
    let job_4 = to_job("http://example.com/4");

    q.enqueue(vec![job_1.clone(), job_2.clone()]);
    q.dequeue(2);
    assert_eq!(q.visited.len(), 2);
    assert!(q.visited.contains(&job_1));
    assert!(q.visited.contains(&job_2));
    q.enqueue(vec![job_3.clone(), job_4.clone()]);
    q.dequeue(2);
    assert_eq!(q.visited.len(), 2);
    assert!(q.visited.contains(&job_3));
    assert!(q.visited.contains(&job_4));
}
