use clap::Parser as ClapParser;
use env_logger::Env;
use log::{info, warn};
use std::{fmt::Debug, hash::Hash, io::Error, sync::Arc};
use tokio::task::JoinSet;
use url_crawler::{
    crawler::{crawl, crawl_seed},
    dependencies::{
        data_store, url_frontier, Dependencies, Deps, DepsConcrete, UrlFrontierOptions,
    },
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

async fn execute<T: Clone + Hash + Eq, U>(args: Args, deps: Deps<T, U>) -> Result<(), Error> {
    let Args { url, workers_n, .. } = args;

    let original_url_parts = Arc::new(url_parts(&url));

    // first thread attempts to crawl the seed url
    let client: HttpFetch = Fetch::new();
    crawl_seed(deps.clone(), client, original_url_parts.clone()).await?;

    let mut tasks = JoinSet::new();

    for _n in 0..workers_n {
        let client: HttpFetch = Fetch::new(); // each worker gets a HTTP client
        let task = tokio::spawn(crawl(deps.clone(), client, original_url_parts.clone()));

        tasks.spawn(task);
    }

    while let Some(_res) = tasks.join_next().await {
        info!("Worker completed");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // If RUST_LOG env is not set, fallback to printing all logs at info-level or above
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli_args = Args::parse();

    info!("Initialising with seed url: {}", cli_args.url);

    let url_frontier = url_frontier(UrlFrontierOptions {
        delay_s: Some(cli_args.delay),
        uri: cli_args.url.clone(),
    });
    let data_store = data_store();

    let deps = Dependencies::new()
        .url_frontier(url_frontier)
        .data_store(data_store)
        .build();

    match execute(cli_args, deps).await {
        Ok(_) => {
            info!("Done");
        }
        Err(e) => {
            warn!("There's been an error: {}", e)
        }
    }
}
