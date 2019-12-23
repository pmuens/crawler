use crate::lib_utils::hash;
use crate::traits::Persist;
use regex::Regex;
use reqwest::Url;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;

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
pub struct Crawling<T>
where
    T: Persist,
{
    persister: Arc<T>,
    url: Url,
    content: Vec<u8>,
    kind: Kind,
}

impl<T> Crawling<T>
where
    T: Persist,
{
    pub fn new(persister: Arc<T>, url: Url, content_type: &str, content: Vec<u8>) -> Self {
        let kind = Self::identify_kind(content_type);
        Crawling {
            persister,
            url,
            content,
            kind,
        }
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

    pub fn write(&self) -> Result<usize, Box<dyn Error>> {
        let has_domain = self.get_domain().is_some();
        let has_file_extension = self.get_file_extension().is_some();
        if has_domain && has_file_extension {
            let domain_prefix = self.get_domain().unwrap();
            let file_extension = self.get_file_extension().unwrap();
            let content = self.content.as_slice();
            let hash = hash(&content);
            let formatted_str = format!("{}-{}{}", domain_prefix, hash, file_extension);
            let content_id = formatted_str.as_str();
            return self.persister.persist(content_id, content);
        }
        Err(Box::from("Failed to write Crawling"))
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

#[cfg(test)]
mod tests {
    use crate::crawling::{Crawling, Kind};
    use crate::traits::Persist;
    use reqwest::Url;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::error::Error;
    use std::str::FromStr;
    use std::sync::Arc;

    struct MockPersister {
        dest: RefCell<HashMap<String, String>>,
    }
    impl Default for MockPersister {
        fn default() -> Self {
            MockPersister {
                dest: RefCell::new(HashMap::<String, String>::new()),
            }
        }
    }
    impl Persist for MockPersister {
        fn persist(&self, content_id: &str, content: &[u8]) -> Result<usize, Box<dyn Error>> {
            let mut dest = self.dest.borrow_mut();
            dest.insert(
                content_id.to_string(),
                String::from_utf8_lossy(content).to_string(),
            );
            Ok(content_id.len() + content.len())
        }
    }

    fn get_url(url: &str) -> Url {
        Url::from_str(url).unwrap()
    }
    fn get_mock_persister() -> Arc<MockPersister> {
        Arc::new(MockPersister::default())
    }

    #[test]
    fn html_crawling_find_urls_some_urls() {
        let url = get_url("http://example.com");
        let persister = get_mock_persister();

        let crawling = Crawling::new(
            persister,
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
        let url = get_url("http://example.com");
        let persister = get_mock_persister();

        let crawling = Crawling::new(
            persister,
            url,
            "text/html",
            b"<html>Hello World</html>".to_vec(),
        );

        assert_eq!(crawling.kind, Kind::Html);
        assert_eq!(crawling.find_urls(), None);
    }

    #[test]
    fn html_crawling_find_urls_single_quotes() {
        let url = get_url("http://example.com");
        let persister = get_mock_persister();

        let crawling = Crawling::new(
            persister,
            url,
            "text/html",
            b"<html><a href='http://google.com'></a></html>".to_vec(),
        );

        assert_eq!(crawling.kind, Kind::Html);
        assert_eq!(crawling.find_urls(), None);
    }

    #[test]
    fn unknown_crawling_find_urls_invalid_urls() {
        let url = get_url("http://example.com");
        let persister = get_mock_persister();

        let crawling = Crawling::new(
            persister,
            url,
            "application/foo",
            b"This is not valid <a href=\"html\">HTML</a>!\"".to_vec(),
        );

        assert_eq!(crawling.kind, Kind::Unknown);
        assert_eq!(crawling.find_urls(), None);
    }

    #[test]
    fn identify_kind_html() {
        let url = get_url("http://example.com");
        let persister = get_mock_persister();

        let crawling = Crawling::new(
            persister,
            url.clone(),
            "text/html",
            b"<!doctype html>Foo Bar</html>".to_vec(),
        );

        assert_eq!(crawling.kind, Kind::Html);
    }

    #[test]
    fn identify_kind_pdf() {
        let url = get_url("http://example.com/foo.pdf");
        let persister = get_mock_persister();

        let crawling = Crawling::new(
            persister,
            url.clone(),
            "application/pdf",
            (&[1, 2, 3]).to_vec(),
        );

        assert_eq!(crawling.kind, Kind::Pdf);
    }

    #[test]
    fn identify_kind_unknown() {
        let url = get_url("http://example.com");
        let persister = get_mock_persister();

        let crawling = Crawling::new(
            persister,
            url,
            "application/foo",
            (&[1, 2, 3, 4, 5, 6]).to_vec(),
        );

        assert_eq!(crawling.kind, Kind::Unknown);
    }

    #[test]
    fn crawling_write() {
        let url = get_url("http://example.com");
        let persister = get_mock_persister();

        let crawling = Crawling::new(persister, url, "text/html", b"Hello World!".to_vec());
        let result = crawling.write();

        let dest_ref = crawling.persister.dest.borrow();

        assert_eq!(result.unwrap(), 49);
        assert_eq!(dest_ref.len(), 1);
        assert_eq!(
            dest_ref
                .get("example.com-12596474995416492747.html")
                .unwrap(),
            "Hello World!"
        );
    }

    #[test]
    fn crawling_get_domain() {
        let url = get_url("http://example.com/foo?bar&baz=qux#some");
        let persister = get_mock_persister();

        let crawling = Crawling::new(persister, url, "text/html", b"Hello World!".to_vec());

        assert_eq!(crawling.get_domain(), Some("example.com"));
    }

    #[test]
    fn crawling_get_file_extension() {
        let url = get_url("http://example.com");
        let persister = get_mock_persister();

        let crawling_html = Crawling::new(
            persister.clone(),
            url.clone(),
            "text/html",
            b"Hello World!".to_vec(),
        );
        let crawling_pdf = Crawling::new(
            persister.clone(),
            url.clone(),
            "application/pdf",
            (&[1, 2, 3]).to_vec(),
        );
        let crawling_unknown = Crawling::new(
            persister.clone(),
            url.clone(),
            "application/foo",
            (&[1, 2, 3]).to_vec(),
        );

        assert_eq!(crawling_html.get_file_extension(), Some(".html"));
        assert_eq!(crawling_pdf.get_file_extension(), Some(".pdf"));
        assert_eq!(crawling_unknown.get_file_extension(), None);
    }
}
