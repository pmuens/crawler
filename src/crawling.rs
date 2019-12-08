use regex::Regex;
use reqwest::Url;
use std::io::Write;
use std::str::FromStr;

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

    pub fn find_urls(&self) -> Option<Vec<Url>> {
        if self.kind == Kind::Html {
            let content = String::from_utf8_lossy(&self.content_raw);
            let mut links: Vec<Url> = vec![];
            for cap in LINK_REGEX.captures_iter(&content) {
                if cap.get(2).is_some() {
                    let mut url = String::from(&cap[2]);
                    if !url.starts_with("http") || !url.starts_with("https") {
                        url = self.url.join(&url).unwrap().to_string();
                    }
                    let parsing = Url::from_str(&url);
                    if parsing.is_ok() {
                        links.push(parsing.unwrap());
                    }
                }
            }
            if !links.is_empty() {
                return Some(links);
            }
        }
        None
    }

    pub fn determine_kind(content_raw: &[u8]) -> Kind {
        let parsing = String::from_utf8_lossy(content_raw);
        if parsing.ends_with("html>")
            || parsing.ends_with("HTML>")
            || parsing.starts_with("<html")
            || parsing.starts_with("<HTML")
            || parsing.starts_with("<!doctype html")
            || parsing.starts_with("<!DOCTYPE html")
        {
            return Kind::Html;
        }
        Kind::Unknown
    }

    pub fn write<T: Write>(&self, sink: &mut T) {
        sink.write_all(&self.content_raw);
    }
}

#[test]
fn html_crawling_find_urls_some_urls() {
    let get_url = |url: &str| Url::from_str(url).unwrap();
    let url = get_url("http://example.com");

    let crawling = Crawling::new(
        url,
        b"<html>\
            Read the <a href=\"news\">News</a>, go back to\
            <a href=\"/home?foo=bar&baz=qux#foo\">Home</a> or visit\
            <a href=\"https://jdoe.com\">Johns Website</a>.\
            <a href=\"mailto:jdoe@example.com\">Contanct Me</a>\
            </html>"
            .to_vec(),
    );

    assert_eq!(crawling.kind, Kind::Html);
    assert_eq!(
        crawling.find_urls(),
        Some(vec![
            get_url("http://example.com/news"),
            get_url("http://example.com/home?foo=bar&baz=qux#foo"),
            get_url("https://jdoe.com"),
            get_url("mailto:jdoe@example.com")
        ])
    );
}

#[test]
fn html_crawling_find_urls_no_urls() {
    let get_url = |url: &str| Url::from_str(url).unwrap();
    let url = get_url("http://example.com");

    let crawling = Crawling::new(url, b"<html>Hello World</html>".to_vec());

    assert_eq!(crawling.kind, Kind::Html);
    assert_eq!(crawling.find_urls(), None);
}

#[test]
fn html_crawling_find_urls_single_quotes() {
    let get_url = |url: &str| Url::from_str(url).unwrap();
    let url = get_url("http://example.com");

    let crawling = Crawling::new(
        url,
        b"<html><a href='http://google.com'></a></html>".to_vec(),
    );

    assert_eq!(crawling.kind, Kind::Html);
    assert_eq!(crawling.find_urls(), None);
}

#[test]
fn unknown_crawling_find_urls_invalid_urls() {
    let get_url = |url: &str| Url::from_str(url).unwrap();
    let url = get_url("http://example.com");

    let crawling = Crawling::new(
        url,
        b"This is not valid <a href=\"html\">HTML</a>!\"".to_vec(),
    );

    assert_eq!(crawling.kind, Kind::Unknown);
    assert_eq!(crawling.find_urls(), None);
}

#[test]
fn determine_kind_html() {
    let url = Url::from_str("http://example.com").unwrap();

    let crawling_1 = Crawling::new(url.clone(), b"<!doctype html>Foo Bar".to_vec());
    assert_eq!(crawling_1.kind, Kind::Html);
    let crawling_2 = Crawling::new(url.clone(), b"<!DOCTYPE html>Foo Bar".to_vec());
    assert_eq!(crawling_2.kind, Kind::Html);
    let crawling_3 = Crawling::new(url.clone(), b"<html>Foo Bar".to_vec());
    assert_eq!(crawling_3.kind, Kind::Html);
    let crawling_4 = Crawling::new(url.clone(), b"<HTML>Foo Bar".to_vec());
    assert_eq!(crawling_4.kind, Kind::Html);
    let crawling_5 = Crawling::new(url.clone(), b"Foo Bar</html>".to_vec());
    assert_eq!(crawling_5.kind, Kind::Html);
    let crawling_6 = Crawling::new(url.clone(), b"Foo Bar</HTML>".to_vec());
    assert_eq!(crawling_6.kind, Kind::Html);
}

#[test]
fn determine_kind_unknown() {
    let url = Url::from_str("http://example.com").unwrap();

    let crawling = Crawling::new(url, (&[1, 2, 3, 4, 5, 6]).to_vec());

    assert_eq!(crawling.kind, Kind::Unknown);
}

#[test]
fn crawling_write() {
    let url = Url::from_str("http://example.com").unwrap();

    let mut sink: Vec<u8> = vec![];

    let crawling = Crawling::new(url, b"Hello World!".to_vec());
    crawling.write(&mut sink);

    assert_eq!(sink, crawling.content_raw);
}
