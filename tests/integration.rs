extern crate crawler;

use crawler::crawler::Crawler;
use crawler::shared;
use crawler::traits::{Fetch, Persist};
use std::sync::Mutex;

#[derive(Clone, Eq, PartialEq, Hash)]
struct MockFetcher;
impl MockFetcher {
    pub fn new() -> Self {
        MockFetcher
    }
}
impl Fetch for MockFetcher {
    fn fetch(&self, _url: &str) -> shared::Result<(String, Vec<u8>)> {
        let content_type = "text/html".to_string();
        let content = br###"
            <html>
                <body>
                    <main>
                        <h1>Welcome</h1>
                        <header>
                            <a href="https://example.com/about">About</a>
                        <header>
                    
                        <h2>The Doe Family</h2>
                        <a href="https://www.doe.com">The Does</a>
                        <a href="http://john.doe.com/about">John Doe</a>
                        <a href="http://jane.doe.com/about">Jane Doe</a>
                    </main>
                    
                    <footer>
                        <a href="https://example.com/imprint">Imprint</a>
                    </footer>
                </body>
            </html>
        "###
        .to_vec();

        Ok((content_type, content))
    }
}

struct MockPersister {
    dest: Mutex<Vec<String>>,
}
impl MockPersister {
    fn new() -> Self {
        MockPersister {
            dest: Mutex::new(Vec::new()),
        }
    }
}
impl Persist for MockPersister {
    fn persist(&self, content_id: &str, _content: &[u8]) -> shared::Result<usize> {
        let mut dest = self.dest.lock().unwrap();
        dest.push(content_id.to_string());
        Ok(content_id.len())
    }
}

#[test]
fn integration() {
    let url = "http://example.com";
    let num_threads: usize = 2;

    let persister = MockPersister::new();
    let fetcher = MockFetcher::new();

    let mut crawler = Crawler::new(persister, fetcher, num_threads);
    let _result = crawler.start(url);

    let persister_ref = crawler.get_persister();
    let persister_content = persister_ref.dest.lock().unwrap();
    let mut expected_content = persister_content.to_vec();
    expected_content.sort();

    // the content our `MockFetcher` returns has 5 urls. Including the starting URL we have a total
    // of 6 URLs the crawler should crawl
    assert_eq!(persister_content.len(), 6);
    assert_eq!(
        expected_content,
        vec![
            "example.com-5364512737893576011.html",
            "example.com-5364512737893576011.html",
            "example.com-5364512737893576011.html",
            "jane.doe.com-5364512737893576011.html",
            "john.doe.com-5364512737893576011.html",
            "www.doe.com-5364512737893576011.html"
        ]
    );
}
