extern crate reqwest;
extern crate url;

use reqwest::Client;
use std::error::Error;
use std::str::FromStr;
use std::time::SystemTime;
use url::Url;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub struct Request {
    url: Url,
    created_at: SystemTime,
}

impl Request {
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let parsed_url = Url::from_str(url)?;
        let req = Request {
            url: parsed_url,
            created_at: SystemTime::now(),
        };
        Ok(req)
    }

    pub fn get(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut resp = CLIENT.get(&self.url.to_string()).send()?;
        if !resp.status().is_success() {
            Err(resp.status().to_string())?;
        }

        let mut buffer: Vec<u8> = vec![];
        resp.copy_to(&mut buffer)?;

        Ok(buffer)
    }
}
