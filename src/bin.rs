use crawler::args::Args;
use crawler::crawler::Crawler;
use crawler::shared::{self, FSPersister, MainFetcher};
use std::env::args;

fn main() {
    run_binary().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}

fn run_binary() -> shared::Result<()> {
    let arguments: Vec<String> = args().collect();
    let args = Args::new(&arguments)?;

    let persister = FSPersister::new(args.out_dir)?;
    let fetcher = MainFetcher::new();

    let mut crawler = Crawler::new(persister, fetcher, args.num_threads);
    crawler.start(args.url)?;

    Ok(())
}
