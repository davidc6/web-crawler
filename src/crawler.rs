use log::{info, warn};
use reqwest::IntoUrl;

use crate::{
    data_store::DataStore,
    dependencies::Deps,
    fetch::Fetch,
    fetch::HttpFetch,
    parser::Parser,
    url::{self, process_url, UrlParts},
    url_frontier::Dequeue,
};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    io::Error,
    sync::Arc,
};

pub async fn crawl_seed<
    T: Hash + Eq + Clone + Debug + Send + Sync + IntoUrl + Display + 'static,
    U: Send + Sync + Debug + 'static,
>(
    deps: Deps<T, U>,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) -> Result<(), Error> {
    let client: HttpFetch = Fetch::new();
    let task = tokio::spawn(crawl(deps.clone(), client, original_url_parts.clone()));
    task.await?;
    Ok(())
}

pub async fn crawl<T: Hash + Eq + Debug + Display + Clone + Send + IntoUrl, U: Debug + 'static>(
    deps: Deps<T, U>,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) {
    let mut deps = deps.write().await;

    loop {
        let Some(current_url) = deps.url_frontier.dequeue().await else {
            return;
        };
        if deps.data_store.has_visited(&current_url) {
            continue;
        }
        let content = http.get(current_url.clone()).await;
        if content.is_err() {
            warn!(
                "Error requesting URL {} - {:?}",
                &current_url,
                content.err()
            );
            continue;
        };
        let content = content.unwrap();
        deps.data_store.add(current_url.clone(), None);
        deps.data_store.visited(&current_url);

        let urls_founds = Parser::new(content).all_links();

        info!("Visited URL: {}", current_url);

        for url in urls_founds {
            let url = process_url(url, &current_url);
            info!("Found URL: {}", url);

            deps.data_store.add(current_url.clone(), Some(url.clone()));
        }
    }
}
