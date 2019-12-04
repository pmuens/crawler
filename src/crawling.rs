extern crate regex;
extern crate url;

use regex::Regex;
use std::error::Error;
use std::io::Write;
use std::str::{from_utf8, FromStr};
use std::time::SystemTime;
use url::Url;

lazy_static! {
    static ref LINK_REGEX: Regex =
        Regex::new(r#"\s*(?i)href\s*=\s*("([^"]*)"|'[^']*'|([^'">\s]+))"#)
            .unwrap_or_else(|_| panic!("Error parsing Regex"));
}

#[derive(PartialEq, Debug)]
pub enum Kind {
    Html,
    Unknown,
}

pub struct Crawling {
    url: Url,
    content_raw: Vec<u8>,
    kind: Kind,
    created_at: SystemTime,
}

impl Crawling {
    pub fn new(url: &str, content_raw: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let parsed_url = Url::from_str(url)?;
        let kind = Self::determine_kind(&content_raw);
        let crawl = Crawling {
            url: parsed_url,
            content_raw,
            kind,
            created_at: SystemTime::now(),
        };
        Ok(crawl)
    }

    pub fn get_all_links(&self) -> Option<Vec<String>> {
        let content = from_utf8(&self.content_raw[..])
            .unwrap_or_else(|_| panic!("Invalid HTML document content"));
        let mut links = vec![];
        for cap in LINK_REGEX.captures_iter(content) {
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

    pub fn determine_kind(content_raw: &[u8]) -> Kind {
        let parsing_attempt = String::from_utf8_lossy(content_raw);
        if parsing_attempt.ends_with("html>") {
            return Kind::Html;
        }
        Kind::Unknown
    }

    pub fn write<T: Write>(&self, sink: &mut T) {
        sink.write_all(&self.content_raw);
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

    let html_crawling = Crawling::new(&url, content_raw).unwrap();

    assert_eq!(
        html_crawling.get_all_links().unwrap(),
        [
            "http://example.com/news".to_string(),
            "http://example.com/home?foo=bar&baz=qux#foo".to_string(),
            "https://jdoe.com".to_string(),
        ]
    );
}

#[test]
fn determine_kind_html() {
    let url = "http://example.com";
    let mut content_raw = vec![];
    content_raw.write_all(b"<html>Foo Bar</html>").unwrap();

    let crawling = Crawling::new(url, content_raw).unwrap();

    assert_eq!(crawling.kind, Kind::Html);
}

#[test]
fn determine_kind_unknown() {
    let url = "http://example.com";
    let mut content_raw = vec![];
    content_raw.write_all(&[1, 2, 3, 4, 5, 6]).unwrap();

    let crawling = Crawling::new(url, content_raw).unwrap();

    assert_eq!(crawling.kind, Kind::Unknown);
}

#[test]
fn crawling_write() {
    let url = "http://example.com";
    let mut content_raw = vec![];
    content_raw.write_all(b"Hello World!").unwrap();

    let mut sink: Vec<u8> = vec![];

    let crawling = Crawling::new(&url, content_raw).unwrap();
    crawling.write(&mut sink);

    assert_eq!(sink, crawling.content_raw);
}
