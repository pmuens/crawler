use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::str::FromStr;
use url::Url;

/// Priority (P1 = Highest)
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Priority {
    P1 = 0,
    P2 = 1,
    P3 = 2,
    P4 = 3,
}

/// BinaryHeap which sorts Jobs to be processed based on their Priority
#[allow(dead_code)]
pub type JobQueue<Job> = BinaryHeap<Job>;

/// A crawling job
#[derive(Debug)]
pub struct Job {
    pub priority: Priority,
    pub url: Url,
}

impl Job {
    pub fn new(url: Url, priority: Priority) -> Self {
        Job { url, priority }
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.priority <= other.priority {
            Some(Ordering::Greater)
        } else if self.priority >= other.priority {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl Eq for Job {}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

#[test]
fn job_queue() {
    let to_url = |url: &str| Url::from_str(url).unwrap();

    let job1 = Job::new(to_url("https://example.com/p1/1"), Priority::P1);
    let job2 = Job::new(to_url("https://example.com/p1/2"), Priority::P1);
    let job3 = Job::new(to_url("https://example.com/p2/1"), Priority::P2);
    let job4 = Job::new(to_url("https://example.com/p2/2"), Priority::P2);
    let job5 = Job::new(to_url("https://example.com/p3/1"), Priority::P3);

    let jobs = vec![&job2, &job5, &job3, &job4, &job1];

    let mut queue = JobQueue::from(jobs);

    assert_eq!(queue.pop(), Some(&job1));
    assert_eq!(queue.pop(), Some(&job2));
    assert_eq!(queue.pop(), Some(&job3));
    assert_eq!(queue.pop(), Some(&job4));
    assert_eq!(queue.pop(), Some(&job5));
}
