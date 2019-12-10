extern crate crawler;

use crawler::run_single_threaded;
use std::env::args;

fn main() {
    if args().len() != 3 {
        println!("Usage: single URL OUT_DIR");
        std::process::exit(1)
    }

    let url = args().nth(1).unwrap();
    let dest = args().nth(2).unwrap();

    run_single_threaded(&url, &dest).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}
