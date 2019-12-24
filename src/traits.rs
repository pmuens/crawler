use crate::error::CrawlerError::{ContentTypeError, RequestError};
use crate::shared;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub trait Persist {
    fn persist(&self, id: &str, url: &str, content: &[u8]) -> shared::Result<usize>;
}

pub trait Fetch {
    fn fetch(&self, url: &str) -> shared::Result<(String, Vec<u8>)> {
        let mut resp = CLIENT.get(url).send()?;
        if !resp.status().is_success() {
            return Err(RequestError(format!("{}", resp.status())));
        }

        if let Some(header) = resp.headers().get(CONTENT_TYPE) {
            let content_type = header.to_str().unwrap().to_string();

            if self.get_content_type_blacklist().is_some()
                && self
                    .get_content_type_blacklist()
                    .unwrap()
                    .into_iter()
                    .any(|t| content_type.contains(t))
            {
                return Err(ContentTypeError(format!(
                    "Blacklisted Content-Type \"{}\" for URL \"{}\"",
                    content_type, &url,
                )));
            }

            let mut buffer: Vec<u8> = vec![];
            resp.copy_to(&mut buffer)?;

            return Ok((content_type, buffer));
        }

        Err(ContentTypeError(format!(
            "Invalid Content-Type for URL \"{}\"",
            &url,
        )))
    }

    fn get_content_type_blacklist<'a>(&self) -> Option<Vec<&'a str>> {
        None
    }
}
