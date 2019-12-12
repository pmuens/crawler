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
        if !self.visited.contains(&value) && !self.queue.contains(&value) {
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

    let job_1 = Job::new(to_url("http://example.com/1")).unwrap();
    let job_2 = Job::new(to_url("http://example.com/1")).unwrap();
    let job_3 = Job::new(to_url("http://example.com/2")).unwrap();
    let job_4 = Job::new(to_url("http://example.com/3")).unwrap();
    let job_5 = Job::new(to_url("http://example.com/3")).unwrap();
    let job_6 = Job::new(to_url("http://example.com/4")).unwrap();
    let job_1_c = job_1.clone(); // /1
    let job_3_c = job_3.clone(); // /2
    let job_4_c = job_4.clone(); // /3
    let job_6_c = job_6.clone(); // /4

    q.enqueue(job_1);
    q.enqueue(job_2);
    q.enqueue(job_3);
    q.enqueue(job_4);
    q.enqueue(job_5);
    q.enqueue(job_6);

    assert_eq!(q.dequeue(), Some(job_1_c));
    assert_eq!(q.dequeue(), Some(job_3_c));
    assert_eq!(q.dequeue(), Some(job_4_c));
    assert_eq!(q.dequeue(), Some(job_6_c));
    assert_eq!(q.dequeue(), None);
}

#[test]
fn job_contains() {
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
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut q = JobQueue::new(3);

    let job_1 = Job::new(to_url("http://example.com/1")).unwrap();
    let job_2 = Job::new(to_url("http://example.com/2")).unwrap();
    let job_3 = Job::new(to_url("http://example.com/3")).unwrap();
    let job_4 = Job::new(to_url("http://example.com/4")).unwrap();
    let job_2_c = job_2.clone(); // /2
    let job_3_c = job_3.clone(); // /3
    let job_4_c = job_4.clone(); // /4

    q.enqueue(job_1);
    q.enqueue(job_2);
    q.enqueue(job_3);
    q.enqueue(job_4);

    assert_eq!(q.dequeue(), Some(job_2_c));
    assert_eq!(q.dequeue(), Some(job_3_c));
    assert_eq!(q.dequeue(), Some(job_4_c));
    assert_eq!(q.dequeue(), None);
}

#[test]
fn job_queue_buffer_visited() {
    let to_url = |url: &str| Url::parse(url).unwrap();
    let mut q = JobQueue::new(2);

    let job_1 = Job::new(to_url("http://example.com/1")).unwrap();
    let job_2 = Job::new(to_url("http://example.com/2")).unwrap();
    let job_3 = Job::new(to_url("http://example.com/3")).unwrap();
    let job_4 = Job::new(to_url("http://example.com/4")).unwrap();

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
