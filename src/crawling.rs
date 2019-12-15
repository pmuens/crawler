use regex::Regex;
use reqwest::Url;
use std::error::Error;
use std::io::Write;
use std::str::FromStr;

lazy_static! {
    static ref LINK_REGEX: Regex =
        Regex::new(r#"\s*(?i)href\s*=\s*("([^"]*)"|'[^']*'|([^'">\s]+))"#)
            .unwrap_or_else(|_| panic!("Error parsing Regex"));
}

#[derive(PartialEq, Debug, Hash)]
pub enum Kind {
    Html,
    Pdf,
    Unknown,
}

#[derive(Hash)]
pub struct Crawling {
    url: Url,
    content: Vec<u8>,
    kind: Kind,
}

impl Crawling {
    pub fn new(url: Url, content_type: &str, content: Vec<u8>) -> Self {
        let kind = Self::identify_kind(content_type);
        Crawling { url, content, kind }
    }

    pub fn find_urls(&self) -> Option<Vec<Url>> {
        if self.kind == Kind::Html {
            let content = String::from_utf8_lossy(&self.content);
            let mut links: Vec<Url> = vec![];
            for cap in LINK_REGEX.captures_iter(&content) {
                if cap.get(2).is_some() {
                    let mut url = String::from(&cap[2]);
                    if !url.starts_with("http") || !url.starts_with("https") {
                        url = self.url.join(&url).unwrap().to_string();
                    }
                    let parsing = Url::from_str(&url);
                    if let Ok(parsing) = parsing {
                        links.push(parsing);
                    }
                }
            }
            if !links.is_empty() {
                return Some(links);
            }
        }
        None
    }

    pub fn identify_kind(content_type: &str) -> Kind {
        if content_type.contains("html") {
            return Kind::Html;
        } else if content_type.contains("pdf") {
            return Kind::Pdf;
        }
        Kind::Unknown
    }

    pub fn write<T: Write>(&self, dest: &mut T) -> Result<(), Box<dyn Error>> {
        dest.write_all(&self.content[..])?;
        Ok(())
    }

    pub fn get_domain(&self) -> Option<&str> {
        self.url.domain()
    }

    pub fn get_file_extension(&self) -> Option<&str> {
        match self.kind {
            Kind::Html => Some(".html"),
            Kind::Pdf => Some(".pdf"),
            Kind::Unknown => None,
        }
    }
}

#[test]
fn html_crawling_find_urls_some_urls() {
    let get_url = |url: &str| Url::from_str(url).unwrap();
    let url = get_url("http://example.com");

    let crawling = Crawling::new(
        url,
        "text/html",
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

    let crawling = Crawling::new(url, "text/html", b"<html>Hello World</html>".to_vec());

    assert_eq!(crawling.kind, Kind::Html);
    assert_eq!(crawling.find_urls(), None);
}

#[test]
fn html_crawling_find_urls_single_quotes() {
    let get_url = |url: &str| Url::from_str(url).unwrap();
    let url = get_url("http://example.com");

    let crawling = Crawling::new(
        url,
        "text/html",
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
        "application/foo",
        b"This is not valid <a href=\"html\">HTML</a>!\"".to_vec(),
    );

    assert_eq!(crawling.kind, Kind::Unknown);
    assert_eq!(crawling.find_urls(), None);
}

#[test]
fn identify_kind_html() {
    let url = Url::from_str("http://example.com").unwrap();

    let crawling = Crawling::new(
        url.clone(),
        "text/html",
        b"<!doctype html>Foo Bar</html>".to_vec(),
    );

    assert_eq!(crawling.kind, Kind::Html);
}

#[test]
fn identify_kind_pdf() {
    let url = Url::from_str("http://example.com/foo.pdf").unwrap();

    let crawling = Crawling::new(url.clone(), "application/pdf", (&[1, 2, 3]).to_vec());

    assert_eq!(crawling.kind, Kind::Pdf);
}

#[test]
fn identify_kind_unknown() {
    let url = Url::from_str("http://example.com").unwrap();

    let crawling = Crawling::new(url, "application/foo", (&[1, 2, 3, 4, 5, 6]).to_vec());

    assert_eq!(crawling.kind, Kind::Unknown);
}

#[test]
fn crawling_write() {
    let url = Url::from_str("http://example.com").unwrap();
    let mut dest: Vec<u8> = vec![];

    let crawling = Crawling::new(url, "text/html", b"Hello World!".to_vec());
    let result = crawling.write(&mut dest);

    assert!(result.is_ok());
    assert_eq!(dest, crawling.content);
}

#[test]
fn crawling_get_domain() {
    let url = Url::from_str("http://example.com/foo?bar&baz=qux#some").unwrap();
    let crawling = Crawling::new(url, "text/html", b"Hello World!".to_vec());

    assert_eq!(crawling.get_domain(), Some("example.com"));
}

#[test]
fn crawling_get_file_extension() {
    let url = Url::from_str("http://example.com").unwrap();

    let crawling_html = Crawling::new(url.clone(), "text/html", b"Hello World!".to_vec());
    let crawling_pdf = Crawling::new(url.clone(), "application/pdf", (&[1, 2, 3]).to_vec());
    let crawling_unknown = Crawling::new(url.clone(), "application/foo", (&[1, 2, 3]).to_vec());

    assert_eq!(crawling_html.get_file_extension(), Some(".html"));
    assert_eq!(crawling_pdf.get_file_extension(), Some(".pdf"));
    assert_eq!(crawling_unknown.get_file_extension(), None);
}
