extern crate crawler;

use crawler::args::Args;
use crawler::run;
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
    run(args.url, args.out_dir, args.num_threads)?;
    Ok(())
}
