# `Crawler`

Multi-threaded Web crawler with support for custom fetching and persisting logic.

## Usage

**NOTE:** See the crates documentation for more info.

### As a binary

The following command will run the crawler with `10` threads, starting with the URL `http://example.com` and storing the visited websites as files in the `./crawlings` directory.

```shell script
cargo run --bin crawler http://example.com ./crawlings 10
```

### As a library

```rust
extern crate crawler;

use crawler::traits::{Fetch, Persist};
use crawler::crawler::Crawler;

// ... trait implementations for `Fetch` and `Persist`

fn main() {
    let url = "http://example.com";
    let num_threads: usize = 2;

    let persister = YourPersister::new();
    let fetcher = YourFetcher::new();

    let mut crawler = Crawler::new(persister, fetcher, num_threads);
    let _result = crawler.start(url);
}
```
