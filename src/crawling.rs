extern crate regex;
extern crate url;

use regex::Regex;
use std::io::Write;
use std::str::{from_utf8, FromStr};
use std::time::SystemTime;
use url::Url;

pub enum Crawling {
    Html(HtmlCrawling),
}

pub struct HtmlCrawling {
    url: Url,
    content_raw: Vec<u8>,
    created_at: SystemTime,
}

impl HtmlCrawling {
    pub fn new(url: &str, content_raw: Vec<u8>) -> Self {
        HtmlCrawling {
            url: Url::from_str(url).unwrap_or_else(|_| panic!("Malformed URL: {}", url)),
            content_raw,
            created_at: SystemTime::now(),
        }
    }

    pub fn get_all_links(&self) -> Option<Vec<String>> {
        let regex = Regex::new(r#"\s*(?i)href\s*=\s*("([^"]*)"|'[^']*'|([^'">\s]+))"#).unwrap();
        let content = from_utf8(&self.content_raw[..])
            .unwrap_or_else(|_| panic!("Invalid HTML document content"));
        let mut links = vec![];
        for cap in regex.captures_iter(content) {
            let mut url = String::from(&cap[2]);
            if !url.starts_with("http") || !url.starts_with("https") {
                url = self.url.join(&url).unwrap().to_string();
            }
            links.push(url);
        }
        if !links.is_empty() {
            return Some(links);
        }
        None
    }
}

#[test]
fn html_crawling_find_links() {
    let url = "http://example.com";
    let mut content_raw = vec![];
    content_raw
        .write_all(
            b"Read the\
            <a href=\"news\">News</a>, go back to\
            <a href=\"/home?foo=bar&baz=qux#foo\">Home</a> or visit\
            <a href=\"https://jdoe.com\">Johns Website</a>.",
        )
        .unwrap();

    let html_crawling = HtmlCrawling::new(&url, content_raw);

    assert_eq!(
        html_crawling.get_all_links().unwrap(),
        [
            "http://example.com/news".to_string(),
            "http://example.com/home?foo=bar&baz=qux#foo".to_string(),
            "https://jdoe.com".to_string(),
        ]
    );
}
