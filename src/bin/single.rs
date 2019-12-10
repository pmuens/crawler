extern crate crawler;

use crawler::args::Args;
use crawler::run_single_threaded;
use std::env::args;
use std::error::Error;

fn main() {
    run().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = args().collect();
    let args = Args::new(&arguments)?;
    run_single_threaded(args.url, args.out_dir)?;
    Ok(())
}
