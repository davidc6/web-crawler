use crate::{
    data_store::DataStore,
    dependencies::DepsConcrete,
    fetch::Fetch,
    fetch::HttpFetch,
    parser::Parser,
    url::{self, filter_url, process_url, UrlParts},
    url_frontier::{Dequeue, Enqueue},
};
use log::{info, warn};
use std::{io::Error, sync::Arc};

pub async fn crawl_seed(
    deps: DepsConcrete,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) -> Result<(), Error> {
    let task = tokio::spawn(crawl(deps.clone(), http, original_url_parts.clone()));
    task.await?;
    Ok(())
}

pub async fn crawl(
    deps: DepsConcrete,
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

            if let Some(url) = filter_url(url, original_url_parts.clone()) {
                if !deps.data_store.has_visited(&url) {
                    deps.url_frontier.enqueue(url);
                }
            };
        }

        info!("--------------------------------------------");
    }
}
