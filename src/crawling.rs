use regex::Regex;
use std::io::Write;
use std::str::FromStr;
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
    pub url: Url,
    pub content_raw: Vec<u8>,
    pub kind: Kind,
}

impl Crawling {
    pub fn new(url: Url, content_raw: Vec<u8>) -> Self {
        let kind = Self::determine_kind(&content_raw);
        Crawling {
            url,
            content_raw,
            kind,
        }
    }

    pub fn get_all_links(&self) -> Option<Vec<String>> {
        if self.kind == Kind::Html {
            let content = String::from_utf8_lossy(&self.content_raw);
            let mut links = vec![];
            for cap in LINK_REGEX.captures_iter(&content) {
                if cap.get(2).is_some() {
                    let mut url = String::from(&cap[2]);
                    if !url.starts_with("http") || !url.starts_with("https") {
                        url = self.url.join(&url).unwrap().to_string();
                    }
                    links.push(url);
                }
            }
            if !links.is_empty() {
                return Some(links);
            }
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
fn html_crawling_get_all_links_links() {
    let url = Url::from_str("http://example.com").unwrap();

    let mut content_raw = vec![];
    content_raw
        .write_all(
            b"<html>\
            Read the <a href=\"news\">News</a>, go back to\
            <a href=\"/home?foo=bar&baz=qux#foo\">Home</a> or visit\
            <a href=\"https://jdoe.com\">Johns Website</a>.\
            </html>",
        )
        .unwrap();

    let crawling = Crawling::new(url, content_raw);

    assert_eq!(crawling.kind, Kind::Html);
    assert_eq!(
        crawling.get_all_links(),
        Some(vec![
            "http://example.com/news".to_string(),
            "http://example.com/home?foo=bar&baz=qux#foo".to_string(),
            "https://jdoe.com".to_string(),
        ])
    );
}

#[test]
fn html_crawling_get_all_links_no_links() {
    let url = Url::from_str("http://example.com").unwrap();

    let mut content_raw = vec![];
    content_raw.write_all(b"<html>Hello World</html>").unwrap();

    let crawling = Crawling::new(url, content_raw);

    assert_eq!(crawling.kind, Kind::Html);
    assert_eq!(crawling.get_all_links(), None);
}

#[test]
fn html_crawling_get_all_links_single_quotes() {
    let url = Url::from_str("http://example.com").unwrap();

    let mut content_raw = vec![];
    content_raw
        .write_all(b"<html><a href='http://google.com'></a></html>")
        .unwrap();

    let crawling = Crawling::new(url, content_raw);

    assert_eq!(crawling.kind, Kind::Html);
    assert_eq!(crawling.get_all_links(), None);
}

#[test]
fn unknown_crawling_get_all_links() {
    let url = Url::from_str("http://example.com").unwrap();
    let mut content_raw = vec![];
    content_raw
        .write_all(b"This is not valid <a href=\"html\">HTML</a>!\"")
        .unwrap();

    let crawling = Crawling::new(url, content_raw);

    assert_eq!(crawling.kind, Kind::Unknown);
    assert_eq!(crawling.get_all_links(), None);
}

#[test]
fn determine_kind_html() {
    let url = Url::from_str("http://example.com").unwrap();
    let mut content_raw = vec![];
    content_raw.write_all(b"<html>Foo Bar</html>").unwrap();

    let crawling = Crawling::new(url, content_raw);

    assert_eq!(crawling.kind, Kind::Html);
}

#[test]
fn determine_kind_unknown() {
    let url = Url::from_str("http://example.com").unwrap();
    let mut content_raw = vec![];
    content_raw.write_all(&[1, 2, 3, 4, 5, 6]).unwrap();

    let crawling = Crawling::new(url, content_raw);

    assert_eq!(crawling.kind, Kind::Unknown);
}

#[test]
fn crawling_write() {
    let url = Url::from_str("http://example.com").unwrap();
    let mut content_raw = vec![];
    content_raw.write_all(b"Hello World!").unwrap();

    let mut sink: Vec<u8> = vec![];

    let crawling = Crawling::new(url, content_raw);
    crawling.write(&mut sink);

    assert_eq!(sink, crawling.content_raw);
}
