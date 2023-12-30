use crate::{
    data_store::DataStore,
    dependencies::Deps,
    fetch::Fetch,
    fetch::HttpFetch,
    url::{self, UrlParts},
    url_frontier::Dequeue,
};
use std::{fmt::Debug, hash::Hash, io::Error, sync::Arc};

pub async fn crawl_seed<
    T: Hash + Eq + Clone + Debug + Send + Sync + 'static,
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

pub async fn crawl<T: Hash + Eq + Debug + Clone + Send, U: Debug + 'static>(
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
    }
}
