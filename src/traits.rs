use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use std::error::Error;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub trait Persist {
    fn persist(&self, content_id: &str, content: &[u8]) -> Result<usize, Box<dyn Error>>;
}

pub trait Fetch {
    fn fetch(&self, url: &str) -> Result<(String, Vec<u8>), Box<dyn Error>> {
        let mut resp = CLIENT.get(url).send()?;
        if !resp.status().is_success() {
            return Err(Box::from(resp.status().to_string()));
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
                let msg = format!(
                    "Blacklisted Content-Type \"{}\" for URL \"{}\"",
                    content_type, &url
                );
                return Err(Box::from(msg));
            }

            let mut buffer: Vec<u8> = vec![];
            resp.copy_to(&mut buffer)?;

            return Ok((content_type, buffer));
        }

        let msg = format!("Invalid Content-Type for URL \"{}\"", &url);
        Err(Box::from(msg))
    }

    fn get_content_type_blacklist<'a>(&self) -> Option<Vec<&'a str>> {
        None
    }
}
