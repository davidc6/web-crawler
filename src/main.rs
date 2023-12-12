use clap::Parser as ClapParser;
use env_logger::Env;
use log::{info, warn};
use std::io::Error;
use url_crawler::dependencies::{Dependencies, Deps};

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
