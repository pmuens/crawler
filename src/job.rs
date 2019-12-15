use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::Url;
use std::collections::{HashSet, VecDeque};
use std::error::Error;

lazy_static! {
    static ref CLIENT: Client = Client::new();
    static ref BLACKLIST_EXTENSIONS: [&'static str; 14] = [
        ".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".tiff", ".ico", ".json", ".woff2",
        ".csv", ".xls", ".xlsx",
        // NOTE: we might want to re-include the following extensions in future releases
        ".xml"
    ];
    static ref BLACKLIST_DOMAINS: [&'static str; 4] = [
        "google", "yahoo", "facebook", "bing"
    ];
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Job {
    url: Url,
}

impl Job {
    pub fn new(url: Url) -> Option<Self> {
        let url_str = url.as_str();
        let blacklisted_extension = BLACKLIST_EXTENSIONS
            .iter()
            .any(|&ext| url_str.ends_with(ext));
        let blacklisted_domain = BLACKLIST_DOMAINS
            .iter()
            .any(|&domain| url_str.contains(format!("{}.", domain).as_str()));
        if blacklisted_extension || blacklisted_domain {
            return None;
        }
        Some(Job { url })
    }

    pub fn get_url(&self) -> Url {
        self.url.to_owned()
    }

    pub fn fetch(&self) -> Result<(String, Vec<u8>), Box<dyn Error>> {
        let mut resp = CLIENT.get(&self.url.to_string()).send()?;
        if !resp.status().is_success() {
            Err(resp.status().to_string())?;
        }

        if let Some(header) = resp.headers().get(CONTENT_TYPE) {
            let content_type = header.to_str().unwrap().to_string();

            let mut buffer: Vec<u8> = vec![];
            resp.copy_to(&mut buffer)?;

            return Ok((content_type, buffer));
        }

        Err(Box::from(format!(
            "Invalid Content-Type for URL \"{}\"",
            self.url
        )))
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
    for ext in BLACKLIST_EXTENSIONS.iter() {
        let formatted = format!("http://example.com/blacklisted.{}", ext);
        let url = formatted.as_str();
        assert!(Job::new(to_url(url)).is_none());
    }
    for domain in BLACKLIST_DOMAINS.iter() {
        let formatted = format!("http://{}.com/foo", domain);
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

    pub fn enqueue(&mut self, job: Job) {
        // remove the oldest element if the queue exceeds the buffer size
        if self.queue.len() == self.buffer {
            self.queue.pop_front();
        }
        // enqueue the new job
        if !self.visited.contains(&job) && !self.queue.contains(&job) {
            self.queue.push_back(job);
        }
    }

    pub fn dequeue(&mut self) -> Option<Job> {
        // clear the whole set if we exceed the buffer size
        if self.visited.len() == self.buffer {
            self.visited.clear();
        }
        // dequeue the job
        if let Some(job) = self.queue.pop_front() {
            // mark as visited when dequeuing it
            self.visited.insert(job.clone());
            return Some(job);
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

    q.enqueue(job_1.clone());
    q.enqueue(job_2.clone());
    q.enqueue(job_3.clone());
    q.enqueue(job_4.clone());
    q.enqueue(job_5.clone());
    q.enqueue(job_6.clone());
    q.enqueue(job_7.clone());

    assert_eq!(q.dequeue(), Some(job_1));
    assert_eq!(q.dequeue(), Some(job_3));
    assert_eq!(q.dequeue(), Some(job_4));
    assert_eq!(q.dequeue(), Some(job_6));
    assert_eq!(q.dequeue(), None);
}

#[test]
fn job_queue_contains() {
    let mut q = JobQueue::new(10);
    let job = Job::new(Url::parse("http://example.com").unwrap()).unwrap();

    q.enqueue(job.clone());
    assert!(q.queue.contains(&job));
    assert!(!q.visited.contains(&job));
    q.dequeue();
    assert!(!q.queue.contains(&job));
    assert!(q.visited.contains(&job));
    // trying to enqueue the same job again
    q.enqueue(job.clone());
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

    q.enqueue(job_1.clone());
    q.enqueue(job_2.clone());
    q.enqueue(job_3.clone());
    q.enqueue(job_4.clone());
    q.enqueue(job_5.clone());

    assert_eq!(q.dequeue(), Some(job_3));
    assert_eq!(q.dequeue(), Some(job_4));
    assert_eq!(q.dequeue(), Some(job_5));
    assert_eq!(q.dequeue(), None);
}

#[test]
fn job_queue_buffer_visited() {
    let to_job = |url: &str| Job::new(Url::parse(url).unwrap()).unwrap();
    let mut q = JobQueue::new(2);

    let job_1 = to_job("http://example.com/1");
    let job_2 = to_job("http://example.com/2");
    let job_3 = to_job("http://example.com/3");
    let job_4 = to_job("http://example.com/4");

    q.enqueue(job_1.clone());
    q.enqueue(job_2.clone());
    q.dequeue();
    q.dequeue();
    assert_eq!(q.visited.len(), 2);
    assert!(q.visited.contains(&job_1));
    assert!(q.visited.contains(&job_2));
    q.enqueue(job_3.clone());
    q.enqueue(job_4.clone());
    q.dequeue();
    q.dequeue();
    assert_eq!(q.visited.len(), 2);
    assert!(q.visited.contains(&job_3));
    assert!(q.visited.contains(&job_4));
}
