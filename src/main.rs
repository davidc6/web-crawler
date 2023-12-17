use clap::Parser as ClapParser;
use env_logger::Env;
use log::{info, warn};
use std::{io::Error, sync::Arc};
use url_crawler::{
    dependencies::{Dependencies, Deps},
    fetch::{Fetch, HttpFetch},
    url::url_parts,
};

#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to crawl
    #[arg(short, long)]
    url: String,

    /// Number of worker threads
    #[arg(short, long, default_value_t = 1)]
    workers_n: u8,

    /// Politeness delay (in seconds) between requests
    #[arg(short, long, default_value_t = 2)]
    delay: u64,

    /// Print data store at the end of the crawl (boolean value)
    #[arg(short, long)]
    print: bool,
}

async fn execute(args: Args, deps: Deps) -> Result<(), Error> {
    let Args { url, workers_n, .. } = args;

    let original_url_parts = Arc::new(url_parts(&url));

    for _n in 0..workers_n {
        let client: HttpFetch = Fetch::new(); // each worker gets a HTTP client
        let is_initial_crawl = true;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // If RUST_LOG env is not set, fallback to printing all logs at info-level or above
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli_args = Args::parse();

    info!("Initialising with seed url: {}", cli_args.url);

    let deps = Dependencies::new();
    match execute(cli_args, deps).await {
        Ok(_) => {
            info!("Done");
        }
        Err(e) => {
            warn!("There's been an error: {}", e)
        }
    }
}
