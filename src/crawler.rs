use crate::{
    dependencies::Deps,
    fetch::Fetch,
    fetch::HttpFetch,
    url::{self, UrlParts},
    url_frontier::Dequeue,
};
use std::{io::Error, sync::Arc};

pub async fn crawl_seed<T: Send + 'static>(
    deps: Deps<T>,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) -> Result<(), Error> {
    let client: HttpFetch = Fetch::new();
    let task = tokio::spawn(crawl(deps.clone(), client, original_url_parts.clone()));
    task.await?;
    Ok(())
}

pub async fn crawl<T: Send>(
    deps: Deps<T>,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) {
    let mut deps = deps.write().await;

    loop {
        let Some(current_url) = deps.url_frontier.dequeue().await else {
            return;
        };
    }
}
