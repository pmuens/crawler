#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate regex;
extern crate reqwest;

#[macro_use]
pub mod logging;
pub mod args;
pub mod bin_utils;
pub mod crawler;
pub mod crawling;
pub mod error;
pub mod job;
pub mod lib_utils;
pub mod shared;
pub mod traits;
