use reqwest::Url;
use std::collections::HashSet;

pub trait Set<T> {
    fn insert(&mut self, value: T) -> bool;
    fn contains(&self, value: &T) -> bool;
}

pub trait Queue<T> {
    fn enqueue(&mut self, value: T);
    fn dequeue(&mut self) -> Option<T>;
}

#[derive(Debug)]
pub struct ProcessedJobs {
    set: HashSet<Job>,
    buffer: u64,
}

impl ProcessedJobs {
    pub fn new(buffer: u64) -> Self {
        ProcessedJobs {
            set: HashSet::<Job>::new(),
            buffer,
        }
    }
}

impl Set<Job> for ProcessedJobs {
    fn insert(&mut self, value: Job) -> bool {
        if self.set.len() == self.buffer as usize {
            self.set.clear();
        }
        self.set.insert(value)
    }

    fn contains(&self, value: &Job) -> bool {
        self.set.contains(value)
    }
}

#[test]
fn processed_jobs() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut p = ProcessedJobs::new(10);

    let job_1 = Job::new(to_url("http://example-1.com"));

    p.insert(job_1);

    assert_eq!(p.set.len(), 1);
    assert_eq!(p.contains(&Job::new(to_url("http://example-1.com"))), true);
}

#[test]
fn processed_jobs_cleanup() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut p = ProcessedJobs::new(2);

    let job_1 = Job::new(to_url("http://example-1.com"));
    let job_2 = Job::new(to_url("http://example-2.com"));
    let job_3 = Job::new(to_url("http://example-3.com"));

    p.insert(job_1);
    p.insert(job_2);
    p.insert(job_3);

    assert_eq!(p.set.len(), 1);
    assert!(p.set.contains(&Job::new(to_url("http://example-3.com"))));
}

#[derive(Debug)]
pub struct JobQueue {
    queue: HashSet<Job>,
    buffer: u64,
}

impl JobQueue {
    pub fn new(buffer: u64) -> Self {
        JobQueue {
            queue: HashSet::<Job>::new(),
            buffer,
        }
    }
}

// NOTE: this implementation isn't respecting the ordering of a traditional Queue
// as we're dealing with a HashSet here which doesn't guarantee ordering
impl Queue<Job> for JobQueue {
    fn enqueue(&mut self, value: Job) {
        if self.queue.len() == self.buffer as usize {
            self.queue.clear();
        }
        self.queue.insert(value);
    }

    fn dequeue(&mut self) -> Option<Job> {
        if self.queue.is_empty() {
            return None;
        }
        // NOTE: we already checked if there are some values so we can safely
        // unwrap the Option here
        let value = self.queue.iter().next().unwrap().to_owned();
        if !self.queue.remove(&value) {
            return None;
        }
        Some(value)
    }
}

#[test]
fn job_queue_enqueue() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut q = JobQueue::new(10);

    let job_1 = Job::new(to_url("http://example-1.com"));
    let job_2 = Job::new(to_url("http://example-1.com"));
    let job_3 = Job::new(to_url("http://example-2.com"));
    let job_4 = Job::new(to_url("http://example-2.com"));

    q.enqueue(job_1);
    q.enqueue(job_2);
    q.enqueue(job_3);
    q.enqueue(job_4);

    assert_eq!(q.queue.len(), 2);
    assert!(q.queue.contains(&Job::new(to_url("http://example-1.com"))));
    assert!(q.queue.contains(&Job::new(to_url("http://example-2.com"))));
}

#[test]
fn job_queue_enqueue_cleanup() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut q = JobQueue::new(2);

    let job_1 = Job::new(to_url("http://example-1.com"));
    let job_2 = Job::new(to_url("http://example-2.com"));
    let job_3 = Job::new(to_url("http://example-3.com"));

    q.enqueue(job_1);
    q.enqueue(job_2);
    q.enqueue(job_3);

    assert_eq!(q.queue.len(), 1);
    assert!(q.queue.contains(&Job::new(to_url("http://example-3.com"))));
}

#[test]
fn job_queue_dequeue() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut q = JobQueue::new(10);

    let job_1 = Job::new(to_url("http://example.com"));

    q.enqueue(job_1);

    assert_eq!(q.dequeue(), Some(Job::new(to_url("http://example.com"))));
    assert_eq!(q.dequeue(), None);
}

/// A crawling job
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Job {
    url: Url,
}

impl Job {
    pub fn new(url: Url) -> Self {
        Job { url }
    }

    pub fn get_url(&self) -> Url {
        self.url.to_owned()
    }
}
