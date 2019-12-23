use reqwest::Error as ReqwestError;
use reqwest::UrlError as ReqwestUrlError;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::Error as IoError;

#[derive(Debug)]
pub enum CrawlerError {
    // "external" errors
    ReqwestError(ReqwestError),
    ReqwestUrlError(ReqwestUrlError),
    IoError(IoError),
    // crate errors
    ParsingError(String),
    FetchingError(String),
    PersistingError(String),
    RequestError(String),
    ContentTypeError(String),
}

impl Error for CrawlerError {}

impl Display for CrawlerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CrawlerError::ReqwestError(ref err) => Display::fmt(err, f),
            CrawlerError::ReqwestUrlError(ref err) => Display::fmt(err, f),
            CrawlerError::IoError(ref err) => Display::fmt(err, f),
            CrawlerError::ParsingError(ref err) => Display::fmt(err, f),
            CrawlerError::FetchingError(ref err) => Display::fmt(err, f),
            CrawlerError::PersistingError(ref err) => Display::fmt(err, f),
            CrawlerError::RequestError(ref err) => Display::fmt(err, f),
            CrawlerError::ContentTypeError(ref err) => Display::fmt(err, f),
        }
    }
}

impl From<ReqwestError> for CrawlerError {
    fn from(e: ReqwestError) -> Self {
        CrawlerError::ReqwestError(e)
    }
}

impl From<ReqwestUrlError> for CrawlerError {
    fn from(e: ReqwestUrlError) -> Self {
        CrawlerError::ReqwestUrlError(e)
    }
}

impl From<IoError> for CrawlerError {
    fn from(e: IoError) -> Self {
        CrawlerError::IoError(e)
    }
}
