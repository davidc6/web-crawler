use crate::{
    dependencies::Deps,
    fetch::Fetch,
    fetch::HttpFetch,
    url::{self, UrlParts}, //     parser::Parser,
                           //     url_frontier::Queue,
};
use std::{io::Error, sync::Arc};

pub async fn crawl_seed(
    deps: Deps,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) -> Result<(), Error> {
    let client: HttpFetch = Fetch::new();
    let task = tokio::spawn(crawl(deps.clone(), client, original_url_parts.clone()));
    task.await?;
    Ok(())
}

pub async fn crawl(
    deps: Deps,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) {
}
