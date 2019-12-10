use reqwest::Client;
use reqwest::Url;
use std::collections::{HashSet, VecDeque};
use std::error::Error;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

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

#[derive(Debug)]
pub struct JobQueue {
    queue: VecDeque<Job>,
    visited: HashSet<Job>,
    buffer: u64,
}

impl JobQueue {
    pub fn new(buffer: u64) -> Self {
        JobQueue {
            queue: VecDeque::<Job>::new(),
            visited: HashSet::<Job>::new(),
            buffer,
        }
    }

    pub fn enqueue(&mut self, value: Job) {
        // remove the oldest element if the queue exceeds the buffer size
        if self.queue.len() == self.buffer as usize {
            self.queue.pop_front();
        }
        if !self.queue.contains(&value) {
            self.queue.push_back(value);
        }
    }

    pub fn dequeue(&mut self) -> Option<Job> {
        if let Some(res) = self.queue.pop_front() {
            // clear the whole set if we reach the buffer size
            if self.visited.len() == self.buffer as usize {
                self.visited.clear();
            }
            // mark as visited when dequeueing it
            self.visited.insert(res.clone());
            return Some(res);
        }
        None
    }
}

#[test]
fn job_queue() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut q = JobQueue::new(10);

    let job_1 = Job::new(to_url("http://example.com/1"));
    let job_2 = Job::new(to_url("http://example.com/1"));
    let job_3 = Job::new(to_url("http://example.com/2"));
    let job_4 = Job::new(to_url("http://example.com/3"));
    let job_5 = Job::new(to_url("http://example.com/3"));
    let job_6 = Job::new(to_url("http://example.com/4"));

    q.enqueue(job_1);
    q.enqueue(job_2);
    q.enqueue(job_3);
    q.enqueue(job_4);
    q.enqueue(job_5);
    q.enqueue(job_6);

    assert_eq!(q.dequeue(), Some(Job::new(to_url("http://example.com/1"))));
    assert_eq!(q.dequeue(), Some(Job::new(to_url("http://example.com/2"))));
    assert_eq!(q.dequeue(), Some(Job::new(to_url("http://example.com/3"))));
    assert_eq!(q.dequeue(), Some(Job::new(to_url("http://example.com/4"))));
    assert_eq!(q.dequeue(), None);
}

#[test]
fn job_queue_buffer_queue() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut q = JobQueue::new(3);

    let job_1 = Job::new(to_url("http://example.com/1"));
    let job_2 = Job::new(to_url("http://example.com/2"));
    let job_3 = Job::new(to_url("http://example.com/3"));
    let job_4 = Job::new(to_url("http://example.com/4"));

    q.enqueue(job_1);
    q.enqueue(job_2);
    q.enqueue(job_3);
    q.enqueue(job_4);

    assert_eq!(q.dequeue(), Some(Job::new(to_url("http://example.com/2"))));
    assert_eq!(q.dequeue(), Some(Job::new(to_url("http://example.com/3"))));
    assert_eq!(q.dequeue(), Some(Job::new(to_url("http://example.com/4"))));
    assert_eq!(q.dequeue(), None);
}

#[test]
fn job_queue_buffer_visited() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut q = JobQueue::new(2);

    let job_1 = Job::new(to_url("http://example.com/1"));
    let job_2 = Job::new(to_url("http://example.com/2"));
    let job_3 = Job::new(to_url("http://example.com/3"));
    let job_4 = Job::new(to_url("http://example.com/4"));

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
