use crate::shared;
use crate::traits::Fetch;
use reqwest::Url;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
use std::sync::Arc;

lazy_static! {
    pub static ref BLACKLIST_CONTENT_TYPES: [&'static str; 15] = [
        "css", "js", "png", "jpg", "jpeg", "gif", "tiff", "ico", "svg", "json", "woff2",
        "csv", "xls", "xlsx",
        // NOTE: we might want to re-include the following content types in future releases
        "xml"
    ];
    pub static ref BLACKLIST_DOMAINS: [&'static str; 7] = [
        "google", "google-analytics", "googleapis", "yahoo", "bing",
        // NOTE: we might want to re-include the following domains in future releases
        "facebook", "twitter"
    ];
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Job<T>
where
    T: Fetch,
{
    fetcher: Arc<T>,
    url: Url,
}

impl<T> Job<T>
where
    T: Fetch,
{
    pub fn new(fetcher: Arc<T>, url: Url) -> Option<Self> {
        let url_str = url.as_str();
        let blacklisted_content_type = BLACKLIST_CONTENT_TYPES
            .iter()
            .any(|&t| url_str.contains(format!(".{}", t).as_str()));
        let blacklisted_domain = BLACKLIST_DOMAINS
            .iter()
            .any(|&domain| url_str.contains(format!("{}.", domain).as_str()));
        if blacklisted_content_type || blacklisted_domain {
            return None;
        }
        Some(Job { fetcher, url })
    }

    pub fn get_url(&self) -> Url {
        self.url.to_owned()
    }

    pub fn fetch(&self) -> shared::Result<(String, Vec<u8>)> {
        self.fetcher.fetch(self.url.as_str())
    }
}

#[cfg(test)]
mod job_tests {
    use crate::job::{Job, BLACKLIST_CONTENT_TYPES, BLACKLIST_DOMAINS};
    use crate::shared;
    use crate::traits::Fetch;
    use reqwest::Url;
    use std::sync::Arc;

    struct MockFetcher;
    impl Default for MockFetcher {
        fn default() -> Self {
            MockFetcher {}
        }
    }
    impl Fetch for MockFetcher {
        fn fetch(&self, _url: &str) -> shared::Result<(String, Vec<u8>)> {
            let content_type = "text/html".to_string();
            let content = vec![1, 2, 3, 4];
            Ok((content_type, content))
        }
    }

    fn create_job(url: &str) -> Option<Job<MockFetcher>> {
        let fetcher = MockFetcher::default();
        Job::new(Arc::new(fetcher), Url::parse(url).unwrap())
    }

    #[test]
    fn job() {
        // test the common cases
        assert!(create_job("http://exampe.com/index").is_some());
        assert!(create_job("http://exampe.com/index.php").is_some());
        assert!(create_job("http://example.com/index.html").is_some());
        assert!(create_job("http://example.com/index.pdf").is_some());
        assert!(create_job("http://example.com/index.ps").is_some());
        assert!(create_job("http://example.com/index.txt").is_some());
        for t in BLACKLIST_CONTENT_TYPES.iter() {
            let formatted = format!("http://example.com/blacklisted.{}", t);
            let url = formatted.as_str();
            assert!(create_job(url).is_none());
        }
        for domain in BLACKLIST_DOMAINS.iter() {
            let formatted = format!("http://{}.com/foo", domain);
            let url = formatted.as_str();
            assert!(create_job(url).is_none());
        }
    }

    #[test]
    fn job_fetch() {
        let job = create_job("http://example.com/foo/bar").unwrap();

        let content_type = "text/html".to_string();
        let content: Vec<u8> = vec![1, 2, 3, 4];

        assert_eq!(job.fetch().unwrap(), (content_type, content));
    }
}

pub struct Queue<T> {
    queue: VecDeque<T>,
    seen: HashSet<T>,
    buffer: usize,
}

impl<T> Queue<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new(buffer: usize) -> Self {
        Queue {
            queue: VecDeque::<T>::new(),
            seen: HashSet::<T>::new(),
            buffer,
        }
    }

    pub fn enqueue(&mut self, value: T) {
        // remove the oldest element if the queue exceeds the buffer size
        if self.queue.len() == self.buffer {
            self.queue.pop_front();
        }
        // enqueue the new value
        if !self.seen.contains(&value) && !self.queue.contains(&value) {
            self.queue.push_back(value);
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        // clear the whole set if we exceed the buffer size
        if self.seen.len() == self.buffer {
            self.seen.clear();
        }
        // dequeue the item
        if let Some(value) = self.queue.pop_front() {
            // mark as seen when dequeuing it
            self.seen.insert(value.clone());
            return Some(value);
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod queue_tests {
    use crate::job::{Job, Queue};
    use crate::traits::Fetch;
    use reqwest::Url;
    use std::sync::Arc;

    #[derive(Eq, PartialEq, Hash, Clone, Debug)]
    struct MockFetcher;
    impl Default for MockFetcher {
        fn default() -> Self {
            MockFetcher {}
        }
    }
    impl Fetch for MockFetcher {}

    fn to_job(url: &str) -> Job<MockFetcher> {
        let fetcher = MockFetcher::default();
        Job::new(Arc::new(fetcher), Url::parse(url).unwrap()).unwrap()
    }

    #[test]
    fn queue() {
        let mut q = Queue::new(10);

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
    fn queue_is_empty() {
        let mut q = Queue::new(10);
        let job = to_job("http://example.com");

        assert_eq!(q.is_empty(), true);
        q.enqueue(job);
        assert_eq!(q.is_empty(), false);
        q.dequeue();
        assert_eq!(q.is_empty(), true);
    }

    #[test]
    fn queue_contains() {
        let mut q = Queue::new(10);
        let job = to_job("http://example.com");

        q.enqueue(job.clone());
        assert!(q.queue.contains(&job));
        assert!(!q.seen.contains(&job));
        q.dequeue();
        assert!(!q.queue.contains(&job));
        assert!(q.seen.contains(&job));
        // trying to enqueue the same job again
        q.enqueue(job.clone());
        assert!(!q.queue.contains(&job));
        assert!(q.seen.contains(&job));
    }

    #[test]
    fn queue_buffer_queue() {
        let mut q = Queue::new(3);

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
    fn queue_buffer_seen() {
        let mut q = Queue::new(2);

        let job_1 = to_job("http://example.com/1");
        let job_2 = to_job("http://example.com/2");
        let job_3 = to_job("http://example.com/3");
        let job_4 = to_job("http://example.com/4");

        q.enqueue(job_1.clone());
        q.enqueue(job_2.clone());
        q.dequeue();
        q.dequeue();
        assert_eq!(q.seen.len(), 2);
        assert!(q.seen.contains(&job_1));
        assert!(q.seen.contains(&job_2));
        q.enqueue(job_3.clone());
        q.enqueue(job_4.clone());
        q.dequeue();
        q.dequeue();
        assert_eq!(q.seen.len(), 2);
        assert!(q.seen.contains(&job_3));
        assert!(q.seen.contains(&job_4));
    }
}
