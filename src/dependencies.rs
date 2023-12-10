use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
struct URLFrontier {}

#[derive(Default)]
struct DataStore {}

#[derive(Default)]
pub struct Dependencies {
    url_frontier: Arc<RwLock<URLFrontier>>,
    data_store: Arc<RwLock<DataStore>>,
}
