use crawler::args::Args;
use crawler::bin_utils::{FSPersister, Fetcher};
use crawler::Crawler;
use std::env::args;
use std::error::Error;

fn main() {
    run_binary().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}

fn run_binary() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = args().collect();
    let args = Args::new(&arguments)?;

    let persister = FSPersister::new(args.out_dir)?;
    let fetcher = Fetcher::new();

    let mut crawler = Crawler::new(persister, fetcher, args.num_threads);
    crawler.start(args.url)?;

    Ok(())
}
