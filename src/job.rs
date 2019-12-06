use reqwest::Url;
use std::collections::HashSet;
use std::str::FromStr;

// TODO: this set needs to be cleaned-up on a recurring basis to reclaim memory during runtime
pub type ProcessedJobs = HashSet<Job>;
pub type JobQueue = HashSet<Job>;

pub trait Queue<T> {
    fn enqueue(&mut self, value: T);
    fn dequeue(&mut self) -> Option<T>;
}

// NOTE: this implementation isn't respecting the ordering of a traditional Queue
// as we're dealing with a HashSet here which doesn't guarantee ordering
impl Queue<Job> for JobQueue {
    fn enqueue(&mut self, value: Job) {
        self.insert(value);
    }

    fn dequeue(&mut self) -> Option<Job> {
        if self.is_empty() {
            return None;
        }
        // NOTE: we already checked if there are some values so we can safely
        // unwrap the Option here
        let value = self.iter().next().unwrap().to_owned();
        if !self.remove(&value) {
            return None;
        }
        Some(value)
    }
}

#[test]
fn job_queue_enqueue() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut queue = JobQueue::new();

    let job_1 = Job::new(to_url("http://example-1.com"));
    let job_2 = Job::new(to_url("http://example-1.com"));
    let job_3 = Job::new(to_url("http://example-2.com"));
    let job_4 = Job::new(to_url("http://example-2.com"));

    queue.enqueue(job_1);
    queue.enqueue(job_2);
    queue.enqueue(job_3);
    queue.enqueue(job_4);

    assert_eq!(queue.len(), 2);
    assert!(queue.contains(&Job::new(to_url("http://example-1.com"))));
    assert!(queue.contains(&Job::new(to_url("http://example-2.com"))));
}

#[test]
fn job_queue_dequeue() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut queue = JobQueue::new();

    let job_1 = Job::new(to_url("http://example.com"));

    queue.enqueue(job_1);

    assert_eq!(
        queue.dequeue(),
        Some(Job::new(to_url("http://example.com")))
    );
    assert_eq!(queue.dequeue(), None);
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
