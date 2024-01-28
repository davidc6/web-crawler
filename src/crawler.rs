use crate::{
    dependencies::{Dependencies, Deps, DepsConcrete},
    fetch::{Fetch, HttpFetch},
    parser::Parser,
    url::{self, filter_url, process_url, UrlParts},
};
use log::{info, warn};
use reqwest::IntoUrl;
use std::{fmt::Display, hash::Hash, io::Error, sync::Arc};

pub async fn crawl_seed<T: Clone + Hash + Eq + IntoUrl + Send + Display + AsRef<str>, U>(
    deps: Dependencies<T, U>,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) -> Result<(), Error> {
    let task = tokio::spawn(crawl(deps, http, original_url_parts.clone()));
    task.await?;
    Ok(())
}

pub async fn crawl<T: Clone + Hash + Eq + IntoUrl + Send + Display + AsRef<str>, U>(
    deps: Dependencies<T, U>,
    http: HttpFetch,
    original_url_parts: Arc<Result<UrlParts, url::Error>>,
) {
    let mut url_frontier_write = deps.url_frontier.write().await;
    let mut data_store = deps.data_store.write().await;

    loop {
        let Some(current_url) = url_frontier_write.dequeue().await else {
            return;
        };

        if data_store.has_visited(&current_url) {
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

        data_store.add(current_url.clone(), None);
        data_store.visited(&current_url);

        let urls_founds = Parser::new(content).all_links();

        info!("Visited URL: {}", current_url);

        for url in urls_founds {
            let url = process_url(url, &current_url);
            info!("Found URL: {}", url);

            data_store.add(current_url.clone(), Some(url.clone()));

            if let Some(url) = filter_url(url, original_url_parts.clone()) {
                if !data_store.has_visited(&url) {
                    url_frontier_write.enqueue(url);
                }
            };
        }

        info!("--------------------------------------------");
    }
}

#[cfg(test)]
mod task_tests {
    use crate::crawler::crawl;
    use crate::data_store::{DataStore, DataStoreEntry};
    use crate::dependencies::{Dependencies, Frontier, MemoryStore};
    use crate::fetch::{Fetch, HttpFetch};
    use crate::url::url_parts;
    use crate::url_frontier::{Dequeue, Enqueue, Queue};
    use async_trait::async_trait;
    use mockall::{mock, predicate, Sequence};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_hrefs(base_uri: &str) -> Vec<String> {
        let url1 = format!("{}/about", &base_uri);
        let url2 = format!("{}/contact", &base_uri);
        let url3 = "http://google.com".to_owned();

        vec![url1, url2, url3]
    }

    fn make_anchors(urls: Vec<String>) -> String {
        urls.into_iter()
            .map(|url| format!("<a href=\"{}\"</a>", url))
            .collect::<Vec<_>>()
            .join("")
    }

    mock!(
        #[derive(Debug)]
        Store<T, U: 'static> {}

        impl<T, U: 'static> DataStore<T, U> for Store<T, U> {
            fn add(&mut self, key: T, value: Option<U>);
            fn visited(&mut self, key: &T);
            fn has_visited(&self, key: &T) -> bool;
            fn exists(&self, key: &T) -> bool;
            fn get<'a>(&'a self, key: &T) -> Option<&'a DataStoreEntry<U>>;
        }
    );

    mock!(
        URLFrontier<T> {}

        impl<T: Send> Queue<T> for URLFrontier<T> {}

        impl<T> Enqueue<T> for URLFrontier<T> {
            fn enqueue(&mut self, value: T);
        }

        #[async_trait]
        impl<T: Send> Dequeue<T> for URLFrontier<T> {
            async fn dequeue(&mut self) -> Option<T>;
        }
    );

    #[tokio::test]
    async fn task_runs_correctly() {
        let listener = std::net::TcpListener::bind("localhost:45367").unwrap();
        let mock_server = MockServer::builder().listener(listener).start().await;

        let port = "45367";
        let main_url = format!("http://localhost:{}", port);
        let hrefs = make_hrefs(&main_url);
        let about_url = hrefs.first().unwrap();
        let contact_url = hrefs.get(1).unwrap();
        let anchors = make_anchors(hrefs.to_vec());

        // First request (/)
        let response = ResponseTemplate::new(200).set_body_string(anchors);
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(response.clone())
            .up_to_n_times(1)
            .expect(1..)
            .mount(&mock_server)
            .await;

        // Second request (/about)
        let response = ResponseTemplate::new(200).set_body_string("".to_owned());
        Mock::given(method("GET"))
            .and(path("/about"))
            .respond_with(response.clone())
            .up_to_n_times(1)
            .expect(1..)
            .mount(&mock_server)
            .await;

        // Third request (/contact)
        let response = ResponseTemplate::new(200).set_body_string("".to_owned());
        Mock::given(method("GET"))
            .and(path("/contact"))
            .respond_with(response.clone())
            .up_to_n_times(1)
            .expect(1..)
            .mount(&mock_server)
            .await;

        let mut url_frontier_mock = MockURLFrontier::new();
        let mut data_store_mock = MockStore::new();
        let mut sequence = Sequence::new();

        // /
        url_frontier_mock
            .expect_dequeue()
            .times(1)
            .return_const(Some(main_url.clone()))
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_has_visited()
            .once()
            .with(predicate::eq(main_url.clone()))
            .return_const(false)
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_add()
            .once()
            .with(predicate::eq(main_url.clone()), predicate::eq(None))
            .returning(|_, _| {})
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_visited()
            .once()
            .with(predicate::eq(main_url.clone()))
            .returning(|_| {})
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_add()
            .once()
            .with(
                predicate::eq(main_url.clone()),
                predicate::eq(Some(about_url.clone())),
            )
            .returning(|_, _| {})
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_has_visited()
            .once()
            .with(predicate::eq(about_url.clone()))
            .returning(|_| false)
            .in_sequence(&mut sequence);

        url_frontier_mock
            .expect_enqueue()
            .with(predicate::eq(about_url.clone()))
            .times(1)
            .returning(|_| {})
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_add()
            .once()
            .with(
                predicate::eq(main_url.clone()),
                predicate::eq(Some(contact_url.clone())),
            )
            .returning(|_, _| {})
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_has_visited()
            .once()
            .with(predicate::eq(contact_url.clone()))
            .returning(|_| false)
            .in_sequence(&mut sequence);

        url_frontier_mock
            .expect_enqueue()
            .with(predicate::eq(contact_url.clone()))
            .once()
            .returning(|_| {})
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_add()
            .once()
            .with(
                predicate::eq(main_url.clone()),
                predicate::eq(Some("http://google.com".to_owned())),
            )
            .returning(|_, _| {})
            .in_sequence(&mut sequence);

        url_frontier_mock
            .expect_enqueue()
            .with(predicate::eq("http://google.com".to_owned()))
            .times(0)
            .returning(|_| {});

        // /about
        url_frontier_mock
            .expect_dequeue()
            .times(1)
            .return_const(Some(about_url.clone()))
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_has_visited()
            .once()
            .with(predicate::eq(about_url.clone()))
            .returning(|_| false)
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_add()
            .once()
            .with(predicate::eq(about_url.clone()), predicate::eq(None))
            .returning(|_, _| {})
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_visited()
            .once()
            .with(predicate::eq(about_url.clone()))
            .returning(|_| {})
            .in_sequence(&mut sequence);

        // /contact
        url_frontier_mock
            .expect_dequeue()
            .times(1)
            .return_const(Some(contact_url.clone()))
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_has_visited()
            .once()
            .with(predicate::eq(contact_url.clone()))
            .returning(|_| false)
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_add()
            .once()
            .with(predicate::eq(contact_url.clone()), predicate::eq(None))
            .returning(|_, _| {})
            .in_sequence(&mut sequence);

        data_store_mock
            .expect_visited()
            .once()
            .with(predicate::eq(contact_url.clone()))
            .returning(|_| {})
            .in_sequence(&mut sequence);

        url_frontier_mock
            .expect_dequeue()
            .times(1)
            .returning(|| None)
            .in_sequence(&mut sequence);

        let client: HttpFetch = Fetch::new();
        let url_parts = Arc::new(url_parts(&main_url));
        let url_frontier = Arc::new(RwLock::new(url_frontier_mock));
        let data_store = Arc::new(RwLock::new(data_store_mock));

        let deps = Dependencies::new()
            .url_frontier(Frontier(url_frontier))
            .data_store(MemoryStore(data_store))
            .build();

        crawl(deps, client, url_parts).await;
    }
}
